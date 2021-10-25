//！ config.rs 负责配置文件的相关操作，主要为读取配置文件和生成默认配置

use std::error::Error;
use std::fs::{self, File};

use serde::{Deserialize, Serialize};
use serde_yaml;

const FILENAME: &str = "config.yml";

/// 配置文件内容
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub server: Server,
    pub deny: DenyConfig,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Server {
    pub auth: Auth,
}

/// 代理验证需要的用户名和密码
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct DenyConfig {
    pub request: RequestDeny,
    pub response: ResponseDeny,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct RequestDeny {
    pub line: RequestLine,
    pub headers: Vec<Header>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct RequestLine {
    pub methods: Vec<String>,
    pub path: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ResponseDeny {
    pub headers: Vec<Header>,
}

impl Config {
    /// 从配置文件中读取内容
    pub fn parse(filepath: &str) -> Result<Config, &str> {
        let f = match File::open(filepath) {
            Ok(f) => f,
            Err(_e) => return Err("can not open file"),
        };
        let config: Config = serde_yaml::from_reader(f).unwrap();
        return Ok(config);
    }

    /// 生成默认的配置文件
    pub fn generate_default() -> Result<Config, Box<dyn Error>> {
        // 默认配置
        let default_config: Config = Config {
            server: Server {
                auth: Auth {
                    username: "".to_string(),
                    password: "".to_string(),
                },
            },
            deny: DenyConfig {
                ..DenyConfig::default()
            },
        };

        // 结构体转换成对应的字符串
        let str = serde_yaml::to_string(&default_config).unwrap();

        fs::write(FILENAME, str)?;

        Ok(default_config)
    }
}

#[test]
fn parse_config_test() {
    // config.yml 在根目录
    assert_eq!(Config::parse("test/not_exist.yml").is_err(), true);

    let config = Config::parse("test/config.yml");
    assert_eq!(config.is_ok(), true);
    let config = config.unwrap();
    assert_eq!(config.server.auth.username, "rust");
    assert_eq!(config.server.auth.password, "proxy");
    assert!(&config.deny.request.headers.len() > &0);
}
