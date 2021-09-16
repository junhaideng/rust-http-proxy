use serde::{Deserialize, Serialize};
use serde_yaml;
use std::error::Error;
use std::fs::{self, File};

const FILENAME: &str = "config.yml";

// 配置文件内容
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    ips: Vec<String>,
    domains: Vec<String>,
    ports: Vec<String>,
    auth: Auth,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    username: String,
    password: String,
}

impl Config {
    // 从配置文件中读取内容
    pub fn parse(filepath: &str) -> Result<Config, &str> {
        let f = match File::open(filepath) {
            Ok(f) => f,
            Err(_e) => return Err("can not open file"),
        };
        let config: Config = serde_yaml::from_reader(f).unwrap();
        return Ok(config);
    }

    // 生产默认的配置文件
    pub fn generate_default() -> Result<Config, Box<dyn Error>> {
        // 默认配置
        let default_config: Config = Config {
            ips: vec![],
            domains: vec![],
            ports: vec![],
            auth: Auth {
                username: String::from("admin"),
                password: String::from("admin"),
            },
        };

        // 结构体转换成对应的字符串
        let str = serde_yaml::to_string(&default_config).unwrap();

        fs::write(FILENAME, str)?;

        Ok(default_config)
    }
}

impl Default for Config {
    fn default() -> Self {
        return Config::generate_default().unwrap();
    }
}

#[test]
fn parse_config_test() {
    assert_eq!(Config::parse("config.yml").is_ok(), true);

    assert_eq!(Config::parse("not_exist.yml").is_err(), true);
}
