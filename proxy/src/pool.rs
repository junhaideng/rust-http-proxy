use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

use crate::filter::filter_response;
use crate::http;
use std::io::{Read, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub enum Message {
    NewStream(TcpStream),
    Terminal,
}

// 线程池
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    // 创建一个线程池
    // 1. 创建通道进行数据的传输
    // 2. 创建worker从通道中获取数据进行处理
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute(&self, stream: TcpStream) -> Result<(), mpsc::SendError<Message>> {
        self.sender.send(Message::NewStream(stream))?;
        Ok(())
    }
}

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

    fn handle_stream(stream: &mut TcpStream) {
        // 读取内容，解析协议
        let mut req = http::parse_request(stream);
        println!("{:?}", req);

        // 找到host
        let mut host = req.headers.get("Host").expect("No host specified").clone();
        if host.contains("443") || req.path.contains("https") {
            stream.shutdown(Shutdown::Both);
            println!("not support https");
            return;
        }
        if !host.contains(":") {
            host = host + ":80";
        }
        let mut client = None;

        let mut socket_addrs = match host.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(e) => {
                println!("to socket addrs failed, host: {}", &host);
                println!("{:?}", "httpbin.org:80".to_socket_addrs().unwrap());
                return;
            }
        };

        // 遍历一遍找到一个可行的socket
        for addr in socket_addrs {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(4)) {
                Ok(mut stream) => {
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
        client
            .write(&req.as_bytes())
            .expect("send http request failed");
        println!("resq: {}", String::from_utf8(req.as_bytes()).unwrap());

        client.flush().expect("flush data failed");

        let mut res = http::parse_response(&mut client);

        if filter_response(&res) {
            println!("filter true");
        } else {
            println!("filter false, 全部数据返回");
            stream.write(&res.as_bytes()).unwrap();
            stream.flush().unwrap();
        }

        // TODO 进行过滤
        println!("\nres: {:?}", res);
        println!("body: {:?}\n", String::from_utf8_lossy(&res.body));
    }

    // fn dial_server(host: &str) {
    //   let client = TcpStream::connect_timeout(&host.parse().unwrap(), Duration::from_secs(10)).expect("Connect failed");
    //   client.wri
    // }
}
