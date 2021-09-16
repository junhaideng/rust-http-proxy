use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

use std::io::{BufRead, Read};
use std::net::TcpStream;
use std::str;

// 支持的方法
#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    HEAD,
}

impl Method {
    fn parse(method: &str) -> Result<Method, &str> {
        if method == "GET" {
            return Ok(Method::GET);
        }
        if method == "POST" {
            return Ok(Method::POST);
        }
        if method == "HEAD" {
            return Ok(Method::HEAD);
        }
        Err("No such method")
    }

    fn to_string(&self) -> String {
        match self {
            Self::GET => String::from("GET"),
            Self::POST => String::from("POST"),
            Self::HEAD => String::from("HEAD"),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

// HTTP/1.0 HTTP/1.1 HTTP/2 HTTP/3
#[derive(Debug, PartialEq, Eq)]
pub enum HttpVersion {
    Http1,
    Http11,
    Http2,
    Http3,
}

impl HttpVersion {
    pub fn to_string(&self) -> String {
        match self {
            Self::Http1 => String::from("HTTP/1.0"),
            Self::Http11 => String::from("HTTP/1.1"),
            Self::Http2 => String::from("HTTP/2"),
            Self::Http3 => String::from("HTTP/3"),
        }
    }

    pub fn parse(version: &str) -> Result<HttpVersion, &str> {
        match version {
            "HTTP/1.0" => Ok(Self::Http1),
            "HTTP/1.1" => Ok(Self::Http11),
            "HTTP/2" => Ok(Self::Http2),
            "HTTP/3" => Ok(Self::Http3),
            _ => Err("No such http version supported"),
        }
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub struct RequestLine {
    method: Method,
    path: String,
    version: HttpVersion,
}

// 请求行解析, 比如GET /hello HTTP/1.1
pub fn parse_requst_header(line: &str) -> Result<RequestLine, &str> {
    let line: Vec<_> = line.split(' ').collect();
    if line.len() != 3 {
        return Err("Request line is not correct");
    }
    let method = Method::parse(line[0])?;
    let path = line[1];
    let version = HttpVersion::parse(line[2])?;
    Ok(RequestLine {
        method: method,
        path: path.to_string(),
        version: version,
    })
}

// 状态转移图 TODO
#[derive(Debug, PartialEq, Eq)]
enum State {
    Init,    // 最开始的状态
    More,    // 处于一行中的内容
    NewLine, // 接收到一个\n
    Return1, // 只接收到一个\r
    Return2, // 连续接收到两个\r，或者\r\n\r
    End,     // 最后接收到两个\r\n\r\n或者\n\n的时候结束
    Invalid, // 非法
}

#[derive(Debug, PartialEq, Eq)]
enum CharType {
    Return,  // \r
    NewLine, // \n
    Others,  // 其他字符
}

fn transform(current: State, input: CharType) -> State {
    match current {
        State::Init => match input {
            CharType::Return => State::Return1,
            _ => State::More,
        },
        State::More => match input {
            CharType::Return => State::Return1,
            CharType::NewLine => State::NewLine,
            _ => State::More,
        },
        State::Return1 => match input {
            CharType::NewLine => State::NewLine,
            _ => State::More,
        },
        State::NewLine => match input {
            CharType::Return => State::Return2,
            CharType::NewLine => State::End,
            _ => State::More,
        },
        State::Return2 => match input {
            CharType::NewLine => State::End,
            _ => State::More,
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
    let line = str::from_utf8(&line).unwrap();
    let res: Vec<&str> = line.splitn(2, ':').collect();
    let res: Vec<&str> = res.iter().map(|s| s.trim()).collect();
    if res.len() == 2 {
        return Ok((String::from(res[0]), String::from(res[1])));
    }
    Err("wrong format")
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    path: String,
    version: HttpVersion,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

// 解析http协议内容
pub fn parse(stream: &mut TcpStream) -> Request {
    // 每次读取一个字节
    let mut buf = [0; 1];
    // 保存每一行的内容，会重复利用
    // 当为body的时候会保存所有的内容，可能会有多行
    let mut writer = Vec::new();

    // 首先读取一行数据，里面是请求行或者响应行
    loop {
        let size = stream.read(&mut buf).unwrap();
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

    let request_header = parse_requst_header(str::from_utf8(&writer.clone()).unwrap()).unwrap();

    let mut state = State::Init;
    writer.clear();

    let mut header = HashMap::new();

    loop {
        let size = stream.read(&mut buf).unwrap();
        if size == 0 {
            break;
        }
        state = transform(state, to_char_type(buf[0]));
        if state == State::End {
            break;
        }

        // \r
        if buf[0] == 13 {
            continue;
        }

        // 10 -> \n
        if buf[0] == 10 {
            let res = split_key_value(writer.clone()).unwrap();

            header.insert(res.0.clone(), res.1.clone());
            writer.clear();
            continue;
        }

        writer.push(buf[0]);
    }

    let length: usize = match header.get("Content-Length") {
        Some(length) => length.parse().unwrap(),
        None => 0,
    };
    let mut body = vec![0; length];

    if length != 0 {
        stream
            .read_exact(&mut body)
            .expect("body is less than Content-Length");
    }

    Request {
        method: request_header.method,
        path: request_header.path,
        version: request_header.version,
        headers: header,
        body: body,
    }
}

#[test]
fn method_test() {
    assert!(Method::parse("POST").is_ok());
    assert!(Method::parse("wrong").is_err());

    assert!(HttpVersion::parse("HTTP/1.1").is_ok());

    assert_eq!(HttpVersion::parse("HTTP/1.1").unwrap(), HttpVersion::Http11);
    assert_eq!(HttpVersion::parse("HTTP/2").unwrap(), HttpVersion::Http2);
    assert_eq!(HttpVersion::parse("HTTP/3").unwrap(), HttpVersion::Http3);
}
