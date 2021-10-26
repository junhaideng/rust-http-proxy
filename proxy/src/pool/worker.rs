use std::io::Write;
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::filter::{FilterRequest, FilterResponse, FilterStatus};
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
                Message::NewStream(mut stream) => {
                    // 处理http请求流数据
                    Self::handle_stream(&mut stream, req_chain.clone(), resp_chain.clone());
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
        stream: &mut TcpStream,
        req_chain: Arc<Vec<FilterRequest>>,
        resp_chain: Arc<Vec<FilterResponse>>,
    ) {
        // 读取内容，解析协议
        let mut req = match http::parse_request(stream) {
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
                    http::forbidden(stream);
                    return;
                }
                FilterStatus::Forward => {}
            }
        }

        // 找到host
        let mut host = req.headers.get("Host").expect("No host specified").clone();
        // 目前不支持HTTPS
        if host.contains("443") || req.path.contains("https") {
            stream
                .shutdown(Shutdown::Both)
                .expect("shutdown stream failed");
            info!("do not support https");
            return;
        }

        if CFG.server.auth.enable {
            // 鉴权
            match req.headers.get("Proxy-Authorization") {
                Some(auth) => {
                    let auth = utils::decode(&(auth[6..]).to_string())
                        .expect("decode authorization failed");
                    // 进行
                    if auth.0.eq(&CFG.server.auth.username) && auth.1.eq(&CFG.server.auth.password)
                    {
                        info!("user {} login", auth.0);
                    } else {
                        http::proxy_auth(stream);
                        return;
                    }
                }
                None => {
                    // 要求输入用户名、密码
                    http::proxy_auth(stream);
                    return;
                }
            }
        }
        if !host.contains(":") {
            host = host + ":80";
        }
        let mut client = None;

        let socket_addrs = match host.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(_e) => {
                println!("to socket addrs failed, host: {}", &host);
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
                Err(_) => continue,
            };
        }

        let mut client = match client {
            Some(stream) => stream,
            None => {
                // 连接到目的服务器失败
                eprintln!("Connect to server failed");
                return;
            }
        };

        // 将客户端发送过来的请求发送到服务端
        client
            .write(&req.as_bytes())
            .expect("send http request failed");

        client.flush().expect("flush data failed");

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
                    http::forbidden(stream);
                    return;
                }
                FilterStatus::Forward => {}
            }
        }

        stream.write(&res.as_bytes()).expect("wirte stream failed");
        stream.flush().expect("flush stream failed");
    }
}
