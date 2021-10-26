use std::process;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config as LogConfig, Root};
use log4rs::encode::pattern::PatternEncoder;

pub fn init() {
    let logfile = match FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}",
        )))
        .build("proxy.log")
    {
        Ok(r) => r,
        Err(e) => {
            println!("build FileAppender failed: {}", e);
            process::exit(-1);
        }
    };

    let config = match LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
    {
        Ok(r) => r,
        Err(e) => {
            println!("build LogConfig failed: {}", e);
            process::exit(-1);
        }
    };

    if let Err(e) = log4rs::init_config(config) {
        println!("init_config failed: {}", e);
        process::exit(-1);
    }
}
