//! http.rs 负责http协议的解析

use log::error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{BufRead, BufReader, Read};

use std::str;

/// HTTP 方法
///
/// 对常见的四种进行封装，其余的未进行封装，
/// 由于转发数据的时候是将之前 HTTP 请求的数据全部转发到目的服务器上
/// 所以不用担心不支持其他的方法
#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    HEAD,
    CONNECT,
    OTHERS(String), // 其他的方法
}

impl Method {
    /// 将HTTP请求的字符串转换成对应的枚举类型
    fn parse(method: &str) -> Method {
        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "HEAD" => Method::HEAD,
            "CONNECT" => Method::CONNECT,
            _ => Method::OTHERS(method.to_string()),
        }
    }

    /// 将枚举类型转换成对应的字符串
    fn to_string(&self) -> String {
        match self {
            Self::GET => String::from("GET"),
            Self::POST => String::from("POST"),
            Self::HEAD => String::from("HEAD"),
            Self::CONNECT => String::from("CONNECT"),
            Self::OTHERS(method) => method.clone(),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

/// HTTP 版本号
///
/// 比如 HTTP/1.0 HTTP/1.1 HTTP/2 HTTP/3
#[derive(Debug, PartialEq, Eq)]
pub enum HttpVersion {
    Http1,
    Http11,
    Http2,
    Http3,
}

impl HttpVersion {
    /// 将字符串转成对应的枚举类型
    pub fn parse(version: &str) -> Result<HttpVersion, &str> {
        match version {
            "HTTP/1.0" => Ok(Self::Http1),
            "HTTP/1.1" => Ok(Self::Http11),
            "HTTP/2" => Ok(Self::Http2),
            "HTTP/3" => Ok(Self::Http3),
            _ => Err("No such http version supported"),
        }
    }

    // 将枚举类型转换成对应的字符串
    pub fn to_string(&self) -> String {
        match self {
            Self::Http1 => String::from("HTTP/1.0"),
            Self::Http11 => String::from("HTTP/1.1"),
            Self::Http2 => String::from("HTTP/2"),
            Self::Http3 => String::from("HTTP/3"),
        }
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

impl Default for HttpVersion {
    fn default() -> Self {
        HttpVersion::Http11
    }
}

/// HTTP 请求行
///
/// 包含三个部分： 请求方法、请求资源路径、HTTP 版本号
#[derive(Debug)]
pub struct RequestLine {
    method: Method,
    path: String,
    version: HttpVersion,
}

// 请求行解析, 比如GET /hello HTTP/1.1
fn parse_request_header(line: &str) -> Result<RequestLine, &str> {
    let line: Vec<_> = line.split(' ').collect();
    if line.len() != 3 {
        return Err("Request line is not correct");
    }
    let method = Method::parse(line[0]);
    let path = line[1];
    let version = HttpVersion::parse(line[2])?;
    Ok(RequestLine {
        method: method,
        path: path.to_string().clone(),
        version: version,
    })
}

/// HTTP 响应行
///
/// 包含三个部分： HTTP 版本号、HTTP 状态码、 HTTP状态码对应的文本
#[derive(Debug)]
pub struct ResponseLine {
    pub version: HttpVersion,
    pub code: u16,
    pub text: String,
}

// 解析 HTTP 响应行
fn parse_response_header(line: &str) -> Result<ResponseLine, &str> {
    let line: Vec<_> = line.splitn(3, ' ').collect();
    if line.len() != 3 {
        return Err("Response line is not correct");
    }
    let version = HttpVersion::parse(line[0])?;

    let code = match line[1].parse() {
        Ok(res) => res,
        Err(_) => {
            return Err("parser http code failed");
        }
    };

    let text = line[2];
    Ok(ResponseLine {
        version: version,
        code: code,
        text: text.to_string(),
    })
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Init,    // 最开始的状态
    More,    // 处于一行中的内容
    NewLine, // 接收到一个\n
    Return1, // 只接收到一个\r
    Return2, // 连续接收到两个\r，或者\r\n\r
    End,     // 最后接收到两个\r\n\r\n或者\n\n的时候结束
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
enum CharType {
    Return,  // \r
    NewLine, // \n
    Others,  // 其他字符
}

/// 状态转移图
//
//                         +----------+
//                         |          |                    others
//       +----------+----->|  invalid |<-----------------------------+
//       |          |      |          |                              |
//       |          |      +----------+                              |
//       |     \r|\n|                                                |
//       |          |                                                |
//       |    +-----+-----+           +-----------+           +------+-----+
//       |    |           |  others   |           |   \r      |            |
//       |    |  init     +---------->|   more    +---------->|   Return1  |
//       |    |           |           |           |           |            |
//       |    +-----------+           +---^--+----+           +------+-----+
//       |                                |  |                       |
//       |                                |  |                       |
// others|                          others|  | \n                    |
//       |                                |  |                       |
//       |                                |  |                       |
//       |       +-----------+        +---+--v-----+                 |
//       |       |           |   \n   |            |       \n        |
//       |       |    End    |<-------+   NewLine  <-----------------+
//       |       |           |        |            |
//       |       +-----^-----+        +-----+------+
//       |             |\n                  |
//       |             |                    |
//       |       +-----+-----+              |
//       |       |           |       \r     |
//       +-------+  Return2  |<-------------+
//               |           |
//               +-----------+
fn transform(current: State, input: CharType) -> State {
    match current {
        State::Init => match input {
            CharType::Return | CharType::NewLine => State::Invalid,
            _ => State::More,
        },
        State::More => match input {
            CharType::Return => State::Return1,
            CharType::NewLine => State::NewLine,
            _ => State::More,
        },
        State::Return1 => match input {
            CharType::NewLine => State::NewLine,
            _ => State::Invalid,
        },
        State::NewLine => match input {
            CharType::Return => State::Return2,
            CharType::NewLine => State::End,
            _ => State::More,
        },
        State::Return2 => match input {
            CharType::NewLine => State::End,
            _ => State::Invalid,
        },
        _ => State::More,
    }
}

fn to_char_type(byte: u8) -> CharType {
    match byte {
        10 => CharType::NewLine,
        13 => CharType::Return,
        _ => CharType::Others,
    }
}
#[test]
fn to_char_type_test() {
    assert_eq!(to_char_type('\n' as u8), CharType::NewLine);
    assert_eq!(to_char_type('\r' as u8), CharType::Return);
    assert_eq!(to_char_type('a' as u8), CharType::Others);
}

fn split_key_value(line: Vec<u8>) -> Result<(String, String), &'static str> {
    let line = match str::from_utf8(&line) {
        Ok(res) => res,
        Err(_) => {
            return Err("convert to &str failed");
        }
    };
    let res: Vec<&str> = line.splitn(2, ':').collect();
    let res: Vec<&str> = res.iter().map(|s| s.trim()).collect();
    if res.len() == 2 {
        return Ok((String::from(res[0]), String::from(res[1])));
    }
    Err("wrong format")
}

/// HTTP 请求
///
/// 代表一次 HTTP 请求的所有数据，包括请求行，请求头部，请求实体内容
#[derive(Debug, Default)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    cache: Vec<u8>,
}

impl Request {
    /// 将请求转换成符合协议规范的字节输出
    pub fn as_bytes(&mut self) -> Vec<u8> {
        if self.cache.len() != 0 {
            return self.cache.clone();
        }
        let ret = '\r' as u8; // 回车
        let newline = '\n' as u8; // 换行
        let colon = ':' as u8; // 冒号
        let space = ' ' as u8; // 空格

        // 数据存入u8数组中
        let mut buf = Vec::new();
        // method
        buf.append(&mut self.method.to_string().as_bytes().to_vec());
        buf.push(space);
        // path
        buf.append(&mut self.path.as_bytes().to_vec());
        buf.push(space);
        // version
        buf.append(&mut self.version.to_string().as_bytes().to_vec());
        buf.push(ret);
        buf.push(newline);

        // headers
        for (key, value) in &self.headers {
            buf.append(&mut key.as_bytes().to_vec());
            buf.push(colon);
            buf.push(space);
            buf.append(&mut value.as_bytes().to_vec());
            buf.push(ret);
            buf.push(newline);
        }
        buf.push(ret);
        buf.push(newline);

        // 实体内容
        buf.append(&mut self.body);
        // 写入缓冲中，下一次调用直接返回
        self.cache = buf.clone();

        buf
    }

    pub fn path(&self) -> String {
        let p = &self.path;
        if p.starts_with("http") {
            return p.clone();
        }
        match self.headers.get("Host") {
            Some(h) => {
                let mut res = h.clone();
                res.push_str(p);
                return res;
            }
            None => {
                return format!("unknown path: {}", p);
            }
        }
    }

    pub fn string(self) -> String {
      let mut res = String::new();
      res += &format!("{} {} {} \n {:?}", self.method, self.path, self.version, self.headers);
      res
    }
}

/// HTTP 响应
///
/// 代表HTTP 响应内容，包括响应行，响应头部，响应体
#[derive(Debug, Default)]
pub struct Response {
    pub version: HttpVersion,
    pub code: u16,
    pub text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    cache: Vec<u8>,
}

impl Response {
    /// 将响应转换成符合协议规范的字节输出
    pub fn as_bytes(&mut self) -> Vec<u8> {
        if self.cache.len() > 0 {
            return self.cache.clone();
        }
        let ret = '\r' as u8;
        let newline = '\n' as u8;
        let colon = ':' as u8;
        let space = ' ' as u8;
        let mut buf = Vec::new();
        // version
        buf.append(&mut self.version.to_string().as_bytes().to_vec());
        buf.push(space);

        // method
        buf.append(&mut self.code.to_string().as_bytes().to_vec());
        buf.push(space);
        // text
        buf.append(&mut self.text.as_bytes().to_vec());
        buf.push(space);
        // text
        buf.push(ret);
        buf.push(newline);

        // headers
        for (key, value) in &self.headers {
            buf.append(&mut key.as_bytes().to_vec());
            buf.push(colon);
            buf.push(space);
            buf.append(&mut value.as_bytes().to_vec());
            buf.push(ret);
            buf.push(newline);
        }
        buf.push(ret);
        buf.push(newline);

        // body
        buf.append(&mut self.body);

        self.cache = buf.clone();
        buf
    }

    pub fn string(self) -> String {
      let mut res = String::new();
      res += &format!("{} {} {} \n {:?}", self.version, self.code, self.text, self.headers);
      res
    }
}

/// 解析 HTTP 协议内容
fn parse(stream: &mut dyn BufRead) -> Result<(HashMap<String, String>, Vec<u8>), &'static str> {
    // 每次读取一个字节
    let mut buf = [0; 1];
    // 数据保存
    let mut writer = Vec::new();
    // 当前的状态
    let mut state = State::Init;
    // 头部数据
    let mut header = HashMap::new();

    loop {
        let size = match stream.read(&mut buf) {
            Ok(s) => s,
            Err(_) => return Err("read stream failed"),
        };
        if size == 0 {
            break;
        }
        state = transform(state, to_char_type(buf[0]));
        if state == State::End {
            break;
        }

        if state == State::Invalid {
            return Err("http content not fits the protocol definition");
        }

        // \r
        if buf[0] == 13 {
            continue;
        }

        // 10 -> \n
        if buf[0] == 10 {
            let res = split_key_value(writer.clone())?;

            header.insert(res.0.clone(), res.1.clone());
            writer.clear();
            continue;
        }

        writer.push(buf[0]);
    }

    let length: usize = match header.get("Content-Length") {
        Some(length) => match length.parse() {
            Ok(res) => res,
            Err(_) => {
                return Err("parse Content-Length failed");
            }
        },
        None => 0,
    };
    let mut body = vec![0; length];

    if length != 0 {
        match stream.read_exact(&mut body) {
            Ok(_) => {}
            Err(_) => {
                return Err("body is less than Content-Length");
            }
        }
    }

    Ok((header, body))
}

pub fn parse_request(stream: &mut dyn Read) -> Result<Request, &str> {
    let mut stream = BufReader::new(stream);

    // 每次读取一个字节
    let mut buf = [0; 1];
    // 保存每一行的内容，会重复利用
    // 当为body的时候会保存所有的内容，可能会有多行
    let mut writer = Vec::new();

    // 首先读取一行数据，里面是请求行或者响应行
    loop {
        let size = match stream.read(&mut buf) {
            Ok(s) => s,
            Err(_) => return Err("read stream failed"),
        };
        if size == 0 {
            break;
        }
        // 10 -> \n
        if buf[0] == 10 {
            break;
        }

        // 13 -> \r
        if buf[0] == 13 {
            continue;
        }
        writer.push(buf[0]);
    }
    // writer.clone()
    let tmp = match str::from_utf8(&writer) {
        Ok(str) => str,
        Err(_err) => {
            return Err("convert Vec<u8> to &[u8] failed");
        }
    };

    let request_header = match parse_request_header(tmp) {
        Ok(line) => line,
        Err(_) => {
            error!("header: {}", tmp);
            return Err("parser request header failed");
        }
    };

    let (header, body) = parse(&mut stream)?;

    Ok(Request {
        method: request_header.method,
        path: request_header.path,
        version: request_header.version,
        headers: header,
        body: body,
        cache: vec![],
    })
}

pub fn parse_response(stream: &mut dyn BufRead) -> Result<Response, &str> {
    // 每次读取一个字节
    let mut buf = [0; 1];
    // 保存每一行的内容，会重复利用
    // 当为body的时候会保存所有的内容，可能会有多行
    let mut writer = Vec::new();

    // 首先读取一行数据，里面是请求行或者响应行
    loop {
        let size = match stream.read(&mut buf) {
            Ok(s) => s,
            Err(_err) => {
                return Err("read stream failed");
            }
        };
        if size == 0 {
            break;
        }
        // 10 -> \n
        if buf[0] == 10 {
            break;
        }

        // 13 -> \r
        if buf[0] == 13 {
            continue;
        }
        writer.push(buf[0]);
    }

    let response_header = parse_response_header(match str::from_utf8(&writer) {
        Ok(res) => res,
        Err(_) => {
            return Err("parse failed");
        }
    });

    let response_header = match response_header {
        Ok(r) => r,
        Err(_) => return Err("parse http response header failed"),
    };

    let (header, body) = parse(stream)?;

    Ok(Response {
        version: response_header.version,
        code: response_header.code,
        text: response_header.text,
        headers: header,
        body: body,
        cache: vec![],
    })
}

#[test]
fn method_test() {
    assert_eq!(Method::parse("POST"), Method::POST);
    assert_eq!(Method::parse("PUT"), Method::OTHERS("PUT".to_string()));

    assert!(HttpVersion::parse("HTTP/1.1").is_ok());

    assert_eq!(HttpVersion::parse("HTTP/1.1").unwrap(), HttpVersion::Http11);
    assert_eq!(HttpVersion::parse("HTTP/2").unwrap(), HttpVersion::Http2);
    assert_eq!(HttpVersion::parse("HTTP/3").unwrap(), HttpVersion::Http3);
}

#[test]
fn parse_request_test() {
    use std::io::Cursor;
    let request_str = "POST /login HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello".as_bytes();
    let mut request_ = Cursor::new(request_str);
    let request = parse_request(&mut request_);
    assert!(request.is_ok());

    let request = request.unwrap();

    assert_eq!(request.path, "/login");
    assert_eq!(request.body, "hello".as_bytes());
    assert!(request.headers.get("Content-Length").is_some());
    assert_eq!(request.headers.get("Content-Length").unwrap(), "5")
}

#[test]
fn parse_response_test() {
    use std::io::Cursor;
    let response_str = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello".as_bytes();
    let mut response = Cursor::new(response_str);
    let response = parse_response(&mut response);
    assert!(response.is_ok());

    let response = response.unwrap();

    assert_eq!(response.body, "hello".as_bytes());
    assert!(response.headers.get("Content-Length").is_some());
    assert_eq!(response.headers.get("Content-Length").unwrap(), "5")
}
