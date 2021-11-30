//！ config.rs 负责配置文件的相关操作，主要为读取配置文件和生成默认配置

use std::error::Error;
use std::fs::{self, File};

use serde::{Deserialize, Serialize};
use serde_json::{self, Result as JsonResult};
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
    pub enable: bool,
    pub username: String,
    pub password: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct DenyConfig {
    #[serde(default)]
    pub request: Vec<Request>,
    pub response: Vec<Response>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Request {
    pub name: String,
    pub rule: RequestDeny,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Response {
    pub name: String,
    pub rule: ResponseDeny,
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
        let config: Config = match serde_yaml::from_reader(f) {
            Ok(r) => r,
            Err(e) => {
                println!("{}", &e);
                return Err("deserialize config file failed");
            }
        };
        return Ok(config);
    }

    /// 生成默认的配置文件
    pub fn generate_default() -> Result<Config, Box<dyn Error>> {
        // 默认配置
        let default_config: Config = Config {
            server: Server {
                auth: Auth {
                    enable: false,
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

    pub fn to_json(&self) -> JsonResult<String> {
        serde_json::to_string(self)
    }
}

#[test]
fn parse_config_test() {
    assert_eq!(Config::parse("test/not_exist.yml").is_err(), true);

    assert_eq!(Config::generate_default().is_err(), false);

    let config = Config::parse("test/config.yml");
    println!("{:?}", &config);
    assert_eq!(config.is_ok(), true);

    let config = config.unwrap();
    assert!(config.server.auth.enable);
    assert_eq!(config.server.auth.username, "rust");
    assert_eq!(config.server.auth.password, "proxy");
}
