use super::header::filter_header;
use super::method::filter_request_method;
use super::path::filter_request_path;
use super::FilterStatus;
use crate::config::{Header, Request};
use crate::http;

// 过滤请求中的头部信息
pub fn filter_request(request_config: &Vec<Request>, request: &http::Request) -> FilterStatus {
    // // 是否被过滤掉
    // let mut flag = true;

    // 多个匹配规则 {name, rule}
    for Request { name, rule } in request_config.iter() {
        let mut flag = true;

        // 比较头部
        // 配置中的文件应该全部被 request 包含
        for Header { key, value } in rule.headers.iter() {
            // 如果配置中的头部没有被包含
            if !filter_header(&request.headers, key, value) {
                flag = false;
                break;
            }
        }
        if !flag {
            continue;
        }

        // 比较方法
        if !filter_request_method(&rule.line.methods, request) {
            continue;
        }

        // 比较路径
        if !filter_request_path(&rule.line.path, request) {
            continue;
        }

        println!("Rejected by rule: {}", name);
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}

#[test]
fn filter_request_test() {
    use crate::config::{Request, RequestDeny, RequestLine};
    use std::collections::HashMap;

    let mut request_config = Vec::new();

    let rule = RequestDeny {
        headers: vec![
            Header {
                key: "Content-Type".to_string(),
                value: "pdf".to_string(),
            },
            Header {
                key: "Host".to_string(),
                value: "www.baidu.com".to_string(),
            },
        ],
        line: RequestLine {
            methods: vec!["POST".to_string(), "GET".to_string()],
            path: vec!["/login".to_string()],
        },
    };

    request_config.push(Request {
        name: "test".to_string(),
        rule: rule,
    });

    let mut h = HashMap::new();
    h.insert("Content-Type".to_string(), "pdf".to_string());

    let mut request = http::Request::default();
    request.headers = h.clone();

    // 因为仅仅包含一个，没有全部包含
    assert_eq!(
        filter_request(&request_config, &request),
        FilterStatus::Forward
    );

    h.insert("Host".to_string(), "www.baidu.com".to_string());
    request.headers = h;

    // 头部全部包含，但是方法和路径不对
    assert_eq!(
        filter_request(&request_config, &request),
        FilterStatus::Forward
    );

    // 头部全部包含, 方法正确，和路径不对
    request.method = http::Method::POST;
    assert_eq!(
        filter_request(&request_config, &request),
        FilterStatus::Forward
    );

    request.path = "/login".to_string();
    assert_eq!(
        filter_request(&request_config, &request),
        FilterStatus::Reject
    );
}
