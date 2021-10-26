use std::net::TcpListener;
use std::process;

use log::error;

use crate::banner;
use crate::pool::ThreadPool;

use super::iptables::init as init_iptables;
use super::log::init as init_log;

const VERSION: &str = "v1.0.0";

/// 代理服务器
pub struct Server {
    pub port: String,
    pub host: String,
    // 监听socket
    listener: TcpListener,
    // pool
    pool: ThreadPool,
}

impl Server {
    /// 创建一个新的 Server
    ///
    /// 可以指定对应的地址，端口，线程池大小
    pub fn new(host: &str, port: &str, pool_size: usize) -> Result<Server, &'static str> {
        // 初始化日志
        init_log();

        let l = match TcpListener::bind(format!("{}:{}", host, port)) {
            Ok(res) => res,
            Err(_) => return Err("bind address failed"),
        };
        let pool = ThreadPool::new(pool_size);

        Ok(Server {
            host: host.to_string(),
            port: port.to_string(),
            listener: l,
            pool: pool,
        })
    }

    // 运行服务器
    // 1. 初始化iptalbes配置，流量进行重定向
    // 2. 开启线程池，进行http响应的处理
    // 3. 返回
    pub fn run(&mut self) -> Result<(), String> {
        banner::print(VERSION);
        println!("run server on {}:{}", self.host, self.port);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.pool.execute(stream) {
                        error!("pool execute failed: {}", e);
                        continue;
                    };
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    }

    // 开启透明代理
    pub fn init_iptables(&self) {
        if let Err(e) = init_iptables(&self.port) {
            println!("init iptables failed, due to: \n{}", e);
            process::exit(-1);
        };
    }
}
