use log::{error, info};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};

use super::message::Message;
use super::worker::Worker;

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

        // 过滤机制
        let mut workers = Vec::with_capacity(size);

        // 创建 worker
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
            ));
        }

        ThreadPool {
            workers: workers,
            sender: sender,
        }
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
        info!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            if let Err(e) = self.sender.send(Message::Terminate) {
                println!("send message failed: {}", e);
                error!("send message failed: {}", e);
            }
        }

        info!("Shutting down all workers.");

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(_) = thread.join() {
                    println!("thread join failed");
                    error!("thread join failed");
                };
            }
        }
    }
}
