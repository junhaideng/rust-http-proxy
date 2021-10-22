use std::collections::HashMap;

use crate::{filter::FilterStatus, http, Config};

/// 头部过滤器
///
/// 对头部的信息进行过滤，比如声明 key: value
/// 那么如果头部包含 key，并且对应的头部值**包含** value
/// 那么会被过滤掉
// 过滤头部，通用方法
fn filter_header(headers: &HashMap<String, String>, key: &str, value: &str) -> FilterStatus {
    match headers.get(key) {
        Some(v) => {
            if v.contains(value) {
                return FilterStatus::Reject;
            }
        }
        None => {}
    }
    FilterStatus::Forward
}

// 过滤请求中的头部信息
pub fn filter_request(config: &Config, request: &http::Request) -> FilterStatus {
    for header in config.deny.response.headers.iter() {
        match filter_header(&request.headers, &header.key, &header.value) {
            FilterStatus::Reject => return FilterStatus::Reject,
            _ => {}
        }
    }
    FilterStatus::Forward
}

// 过滤响应中的头部信息
pub fn filter_response(config: &Config, response: &http::Response) -> FilterStatus {
    for header in config.deny.response.headers.iter() {
        match filter_header(&response.headers, &header.key, &header.value) {
            FilterStatus::Reject => return FilterStatus::Reject,
            _ => {}
        }
    }
    FilterStatus::Forward
}

#[test]
fn filter_test() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer hello".to_string());
    headers.insert("Host".to_string(), "www.baidu.com".to_string());

    assert_eq!(
        filter_header(&headers, "Content-Type", "json"),
        FilterStatus::Reject
    );

    assert_eq!(
        filter_header(&headers, "Host", "www.google.com"),
        FilterStatus::Forward
    );
}
