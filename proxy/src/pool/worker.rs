use std::io::{self, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self};
use std::time::Duration;

use crate::config::Config;
use crate::filter::{FilterRequest, FilterResponse, FilterStatus};
use crate::http::Method;
use crate::{http, utils};

use super::message::Message;
use log::{error, info};

lazy_static! {
    static ref CFG: Config = Config::parse("config.yml").expect("parse config.yml failed");
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // 创建 worker 接收数据进行处理
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        req_chain: Arc<Vec<FilterRequest>>,
        resp_chain: Arc<Vec<FilterResponse>>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("require lock failed")
                .recv()
                .expect("receive message failed");

            match message {
                Message::NewStream(stream) => {
                    // 处理http请求流数据
                    Self::handle_stream(stream, req_chain.clone(), resp_chain.clone());
                }
                // 结束
                Message::Terminate => {
                    info!("worker {} terminate ...", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }

    // 处理 HTTP 连接
    fn handle_stream(
        mut stream: TcpStream,
        req_chain: Arc<Vec<FilterRequest>>,
        resp_chain: Arc<Vec<FilterResponse>>,
    ) {
        // 读取内容，解析协议
        let mut req = match http::parse_request(&mut stream) {
            Ok(req) => req,
            Err(err) => {
                error!("parser request failed: {}", err);
                return;
            }
        };

        // 过滤请求
        for request in req_chain.iter() {
            match request(&CFG, &req) {
                FilterStatus::Reject => {
                    info!("reject {:?}", &req);
                    http::forbidden(&mut stream);
                    return;
                }
                FilterStatus::Forward => {}
            }
        }

        // 找到host
        let mut host = match req.headers.get("Host") {
            Some(s) => s.clone(),
            None => {
                error!("No host specified: {:?}", req);
                return;
            }
        };

        // // 目前不支持HTTPS
        // if host.contains("443") || req.path.contains("https") {
        //     warn!("do not support https: {}", req.path);
        //     http::not_support_https(stream);
        //     return;
        // }

        let mut auth = (String::new(), String::new());
        if CFG.server.auth.enable {
            // 鉴权
            match req.headers.get("Proxy-Authorization") {
                Some(a) => {
                    auth = match utils::decode(&(a[6..]).to_string()) {
                        Ok(res) => res,
                        Err(e) => {
                            error!("decode authorization failed: {}", e);
                            return;
                        }
                    };
                    // 进行
                    if auth.0.eq(&CFG.server.auth.username) && auth.1.eq(&CFG.server.auth.password)
                    {
                    } else {
                        http::proxy_auth(&mut stream);
                        return;
                    }
                }
                None => {
                    // 要求输入用户名、密码
                    http::proxy_auth(&mut stream);
                    return;
                }
            }
        }
        println!("{:?}", req);
        if !host.contains(":") {
            host = host + ":80";
        }
        let mut client = None;

        let socket_addrs = match host.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(_e) => {
                error!("convert to socket addrs failed, host: {}", &host);
                return;
            }
        };

        // 遍历一遍找到一个连接成功的 TcpStream
        for addr in socket_addrs {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(4)) {
                Ok(stream) => {
                    client = Some(stream);
                    break;
                }
                Err(err) => {
                    info!("connect to socket failed: {}, try another", err);
                    continue;
                }
            };
        }

        let mut client = match client {
            Some(stream) => stream,
            None => {
                // 连接到目的服务器失败
                error!("Connect to server failed");
                return;
            }
        };

        // 进行 tunnel
        if req.method == Method::CONNECT {
            http::http_status_ok(&mut stream);

            if let Err(e) = stream.set_nonblocking(true) {
                error!("set client stream nonblocking falied: {}", e);
                return;
            };

            if let Err(e) = client.set_nonblocking(true) {
                error!("set client stream nonblocking failed: {}", e);
            };
            let pipes = [(&stream, &client), (&client, &stream)];

            loop {
                for (mut reader, mut writer) in pipes.iter() {
                    match io::copy(&mut reader, &mut writer) {
                        Ok(s) => {
                            // println!("{}", &s);
                            if s == 0 {
                                println!("close");
                                return;
                            }
                        }
                        Err(e) => {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                println!("{}", &e);
                                return;
                            }
                            continue;
                        }
                    }
                }
            }
        }

        // 将客户端发送过来的请求发送到服务端
        if let Err(e) = client.write(&req.as_bytes()) {
            error!("send http request failed: {}", e);
            return;
        }

        if let Err(e) = client.flush() {
            error!("flush data failed: {}", e);
            return;
        }
        let mut client = BufReader::new(client);
        // 解析收到的 HTTP 响应
        let mut res = match http::parse_response(&mut client) {
            Ok(res) => res,
            Err(err) => {
                error!("parse response failed: {}", err);
                return;
            }
        };

        // filter request
        for request in resp_chain.iter() {
            match request(&CFG, &res) {
                FilterStatus::Reject => {
                    info!("reject response: {:?}", &res);
                    http::forbidden(&mut stream);
                    return;
                }
                FilterStatus::Forward => {}
            }
        }

        if CFG.server.auth.enable {
            info!("user `{}` visited  {}", auth.0, req.path());
        } else {
            info!("visited {}", req.path());
        }

        if let Err(e) = stream.write(&res.as_bytes()) {
            error!("wirte stream failed: {}", e);
            return;
        };

        if let Err(e) = stream.flush() {
            error!("flush stream failed: {}", e);
            return;
        };
    }
}
