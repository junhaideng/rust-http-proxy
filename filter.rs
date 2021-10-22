use std::collections::HashMap;
use crate::config::Config;
use crate::http;

/// 过滤状态
///
/// 被过滤还是进行转发代理
#[derive(Debug, PartialEq, Eq)]
pub enum FilterStatus {
    Forward,
    Reject,
}

/// 对客户端的请求进行过滤
pub type FilterRequest = fn(config: &Config, request: &http::Request) -> FilterStatus;

/// 对服务端返回的响应进行过滤
pub type FilterResponse = fn(config: &Config, response: &http::Response) -> FilterStatus;

/// 头部过滤器
///
/// 对头部的信息进行过滤，比如声明 key: value
/// 那么如果头部包含 key，并且对应的头部值**包含** value
/// 那么会被过滤掉
pub struct HeadersFilter {}

impl HeadersFilter {
    // 过滤头部，通用方法
    fn filter_header(headers: &HashMap<String, String>, key: &str, value: &str) -> FilterStatus {
        match headers.get(key) {
            Some(v) => {
                if v.contains(value) {
                    return FilterStatus::Reject;
                }
            }
            _ => return FilterStatus::Forward,
        }
        FilterStatus::Forward
    }

    // 过滤请求中的头部信息
    pub fn filter_request(config: &Config, request: &http::Request) -> FilterStatus {
        for header in config.deny.response.headers.iter() {
            match Self::filter_header(&request.headers, &header.key, &header.value) {
                FilterStatus::Reject => return FilterStatus::Reject,
                _ => {}
            }
        }
        FilterStatus::Forward
    }

    // 过滤响应中的头部信息
    pub fn filter_response(config: &Config, response: &http::Response) -> FilterStatus {
        for header in config.deny.response.headers.iter() {
            match Self::filter_header(&response.headers, &header.key, &header.value) {
                FilterStatus::Reject => return FilterStatus::Reject,
                _ => {}
            }
        }
        FilterStatus::Forward
    }
}

pub struct MethodFilter {}

impl MethodFilter {
    /// 过滤请求方法
    pub fn filter_request_method(config: &Config, request: &http::Request) -> FilterStatus {
        let methods = &config.deny.request.line.methods;
        let req_method = request.method.to_string();

        if methods.contains(&req_method) {
            return FilterStatus::Reject;
        }
        FilterStatus::Forward
    }
}

pub struct PathFilter {}

impl PathFilter {
    pub fn filter_request_path(config: &Config, request: &http::Request) -> FilterStatus {
        let path = &config.deny.request.line.path;
        if path.contains(&request.path) {
            return FilterStatus::Reject;
        }
        FilterStatus::Forward
    }
}

#[test]
fn filter_test() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer hello".to_string());
    headers.insert("Host".to_string(), "www.baidu.com".to_string());

    assert_eq!(
        HeadersFilter::filter_header(&headers, "Content-Type", "json"),
        FilterStatus::Reject
    );

    assert_eq!(
        HeadersFilter::filter_header(&headers, "Host", "www.google.com"),
        FilterStatus::Forward
    );
}
