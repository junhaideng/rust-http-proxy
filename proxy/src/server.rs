use log::info;
use std::error::Error;
use std::net::TcpListener;

use crate::banner;
use crate::config::Config;
use crate::pool::ThreadPool;

use super::iptables::init as init_iptables;
use super::log::init as init_log;

lazy_static! {
    static ref CFG: Config = Config::parse("config.yml").unwrap();
}

const VERSION: &str = "v1.0.0";

/// 代理服务器
pub struct Server {
    port: String,
    host: String,
    // 监听socket
    listener: TcpListener,
    // pool
    pool: ThreadPool,
}

impl Server {
    /// 创建一个新的 Server
    ///
    /// 可以指定对应的地址，端口，线程池大小
    pub fn new(host: &str, port: &str, pool_size: usize) -> Server {
        // 初始化日志
        init_log();
        // init_iptables(port).unwrap();

        let l = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        let pool = ThreadPool::new(pool_size);

        Server {
            host: host.to_string(),
            port: port.to_string(),
            listener: l,
            pool: pool,
        }
    }

    // 运行服务器
    // 1. 初始化iptalbes配置，流量进行重定向
    // 2. 开启线程池，进行http响应的处理
    // 3. 返回
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        banner::print(VERSION);
        info!("run server on {}:{}", self.host, self.port);
        // Iptable::init(&self.port).unwrap();

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.pool.execute(stream).expect("execute failed");
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }

    // 开启透明代理
    pub fn init_iptables(&self) {
        init_iptables(&self.port).expect("init iptables failed, check your permissions！");
    }
}
