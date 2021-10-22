use crate::config::Config;
use crate::filter::{FilterRequest, FilterResponse};
use crate::pool::ThreadPool;
use std::error::Error;
use std::net::TcpListener;
use std::time::SystemTime;
use std::vec;

lazy_static! {
    static ref CFG: Config = Config::parse("config.yml").unwrap();
}

const VERSION: &str = "1.0.0";

/// 代理服务器
pub struct Server {
    port: String,
    host: String,
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
    request_filter_chain: Vec<FilterRequest>,
    response_filter_chain: Vec<FilterResponse>,
}

impl Server {
    /// 创建一个新的 Server
    ///
    /// 可以指定对应的地址，端口，线程池大小
    pub fn new(host: &str, port: &str, pool_size: usize) -> Server {
        let l = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        let pool = ThreadPool::new(pool_size);

        Server {
            host: host.to_string(),
            port: port.to_string(),
            listener: l,
            count: 0,
            start: SystemTime::now(),
            reject: 0,
            pool: pool,
            pool_size: pool_size,
            request_filter_chain: vec![],
            response_filter_chain: vec![],
        }
    }

    // 运行服务器
    // 1. 初始化iptalbes配置，流量进行重定向
    // 2. 开启线程池，进行http响应的处理
    // 3. 返回
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("run server on {}:{}", self.host, self.port);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.count += 1; 
                    println!("get connection stream");
                    self.pool.execute(stream).expect("execute failed");
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }

    // 添加请求过滤器
    pub fn add_request_filter(&mut self, f: FilterRequest) {
      self.request_filter_chain.push(f);
    }

    // 添加响应过滤器
    pub fn add_response_filter(&mut self, f: FilterResponse){
      self.response_filter_chain.push(f);
    }
}
