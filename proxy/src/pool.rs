use std::io::Read;
use std::net::{Shutdown, TcpStream};

use crate::http;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

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

                    // 读取内容
                    let req = http::parse(&mut stream);
                    println!("{:?}", req);
                    stream.shutdown(Shutdown::Both);
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

    fn handle_stream(mut stream: &TcpStream) {
        // let mut content = String::new();
        // 读取内容
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(size) => {
                    if size == 0 {
                        println!("read 0 bytes, stop to read");
                        stream.shutdown(Shutdown::Both).unwrap();
                        break;
                    }
                    // content += str::from_utf8(&buf[..size]).unwrap();
                }
                Err(e) => {
                    eprintln!("failed: {}", e);
                }
            };
        }

        // let first_line = content.lines().next().unwrap();
        // let request_header = http::parse_requst_header(first_line).unwrap();
        // println!("request header: {:?}", request_header);
    }
}
