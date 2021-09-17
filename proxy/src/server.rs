use crate::pool::ThreadPool;
use std::error::Error;
use std::net::TcpListener;
use std::time::SystemTime;
use time;

const VERSION: &str = "1.0.0";

pub struct Server {
    // 监听socket
    listener: TcpListener,
    // 多少个链接
    count: u64,
    // 开始运行时间
    start: SystemTime,
    // 拒绝了多少个请求
    reject: u64,
    // pool
    pool: ThreadPool,
    // 线程池大小
    pool_size: usize,
}

impl Server {
    pub fn new(host: &str, port: &str, pool_size: usize) -> Server {
        let l = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        let pool = ThreadPool::new(pool_size);

        Server {
            listener: l,
            count: 0,
            start: SystemTime::now(),
            reject: 0,
            pool: pool,
            pool_size: pool_size,
        }
    }

    // 运行服务器
    // 1. 初始化iptalbes配置，流量进行重定向
    // 2. 开启线程池，进行http响应的处理
    // 3. 返回
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("{:?}: server start to run", SystemTime::now());

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("get connection stream");
                    self.pool.execute(stream).expect("execute failed");
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }
}
