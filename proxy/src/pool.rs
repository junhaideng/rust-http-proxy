use std::net::{Shutdown, TcpStream, ToSocketAddrs};

use crate::filter::filter_response;
use crate::{http, utils};
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 消息传递
///
/// 程序退出的时候会发送 Terminal 消息，线程池中的线程一个一个进行关闭
pub enum Message {
    NewStream(TcpStream),
    Terminal,
}

/// 线程池
///
/// 负责 HTTP 请求的代理服务，不是通用的线程池
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// 创建一个线程池
    ///
    /// 1. 创建通道进行数据的传输
    /// 2. 创建 worker 从通道中获取数据进行处理
    pub fn new(size: usize) -> ThreadPool {
        // size 一定要大于0
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        // 创建 worker
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// 将 stream 发送给 worker 进行处理
    pub fn execute(&self, stream: TcpStream) -> Result<(), mpsc::SendError<Message>> {
        self.sender.send(Message::NewStream(stream))?;
        Ok(())
    }
}

// 线程池的销毁
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("shutdowing worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // 创建 worker 接收数据进行处理
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewStream(mut stream) => {
                    println!("worker {} recv stream", id);

                    // 处理http请求流数据
                    Self::handle_stream(&mut stream);
                    stream
                        .shutdown(Shutdown::Both)
                        .expect("shutdown stream failed");
                }
                // 结束
                Message::Terminal => {
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
    fn handle_stream(stream: &mut TcpStream) {
        // 读取内容，解析协议
        let mut req = http::parse_request(stream);
        println!("{:?}", req);

        // 找到host
        let mut host = req.headers.get("Host").expect("No host specified").clone();
        // 目前不支持HTTPS
        if host.contains("443") || req.path.contains("https") {
            stream.shutdown(Shutdown::Both);
            println!("not support https");
            return;
        }
        match req.headers.get("Proxy-Authorization") {
            // TODO: 进行密码验证
            Some(auth) => {
                let auth = utils::decode(&(auth[6..]).to_string()).unwrap();
                println!("auth: {:?}", auth);
            }
            None => {
                // 要求输入密码
                stream.write("HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic\r\n\r\n".as_bytes()).unwrap();
                stream.shutdown(Shutdown::Both).unwrap();
                println!("need authorize");
                return;
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

        // 连接到目的服务器失败
        if client.is_none() {
            eprintln!("Connect to server failed");
            return;
        }

        let mut client = client.unwrap();

        // 将客户端发送过来的请求发送到服务端
        client
            .write(&req.as_bytes())
            .expect("send http request failed");
        println!("resq: {}", String::from_utf8(req.as_bytes()).unwrap());

        client.flush().expect("flush data failed");

        // 解析收到的 HTTP 响应
        let mut res = http::parse_response(&mut client);

        if filter_response(&res) {
            // TODO： 应该被过滤掉
            println!("filter true");
        } else {
            println!("filter false, 全部数据返回");
            stream.write(&res.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        println!("\nres: {:?}", res);
        println!("body: {:?}\n", String::from_utf8_lossy(&res.body));
    }
}
