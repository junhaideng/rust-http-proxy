use log::info;
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};

use crate::filter::header::{filter_request, filter_response};
use crate::filter::method::filter_request_method;
use crate::filter::path::filter_request_path;
use crate::filter::FilterStatus;
use crate::http;
use crate::Config;

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
        let request_filter_chain = vec![
            filter_request as fn(&Config, &http::Request) -> FilterStatus,
            filter_request_path,
            filter_request_method,
        ];

        let response_filter_chain =
            vec![filter_response as fn(&Config, &http::Response) -> FilterStatus];

        let mut workers = Vec::with_capacity(size);

        let req_chain = Arc::new(request_filter_chain);
        let res_chain = Arc::new(response_filter_chain);
        // 创建 worker
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&req_chain),
                Arc::clone(&res_chain),
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
            self.sender
                .send(Message::Terminate)
                .expect("send message failed");
        }

        info!("Shutting down all workers.");

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().expect("thread join failed");
            }
        }
    }
}
