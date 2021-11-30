use super::header::filter_header;
use super::FilterStatus;
use crate::config::{Header, Response};
use crate::http;

// 过滤响应
pub fn filter_response(response_config: &Vec<Response>, response: &http::Response) -> FilterStatus {
    for Response { name, rule } in response_config.iter() {
        let mut flag = true;
        // 如果响应的头部没有全部包含配置中的头部
        for Header { key, value } in rule.headers.iter() {
            // 如果配置中的头部没有被包含
            if !filter_header(&response.headers, key, value) {
                flag = false;
                break;
            }
        }
        if !flag {
            continue;
        }
        println!("Rejected by rule: {}", name);
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}

#[test]
fn filter_response_test() {
    use crate::config::ResponseDeny;
    use std::collections::HashMap;

    let mut response_config = Vec::new();

    let rule = ResponseDeny {
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
    };

    response_config.push(Response {
        name: "test".to_string(),
        rule: rule,
    });

    let mut h = HashMap::new();
    h.insert("Content-Type".to_string(), "pdf".to_string());

    let mut response = http::Response::default();
    response.headers = h.clone();

    // 因为仅仅包含一个，没有全部包含
    assert_eq!(
        filter_response(&response_config, &response),
        FilterStatus::Forward
    );

    h.insert("Host".to_string(), "www.baidu.com".to_string());
    response.headers = h;

    // 全部包含
    assert_eq!(
        filter_response(&response_config, &response),
        FilterStatus::Reject
    );
}
