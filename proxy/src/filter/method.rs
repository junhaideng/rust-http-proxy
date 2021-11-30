//! 请求方法过滤

use crate::http;

// 如果请求中的方法被 methods 中含有,则应该过滤掉
pub fn filter_request_method(methods: &Vec<String>, request: &http::Request) -> bool {
    let req_method = request.method.to_string();
    if methods.contains(&req_method) {
        return true;
    }
    false
}

#[test]
fn filter_request_method_test() {
    let mut methods = Vec::new();
    let mut request = http::Request::default();

    methods.push("POST".to_string());
    assert!(!filter_request_method(&methods, &request),);

    request.method = http::Method::POST;
    assert!(filter_request_method(&methods, &request));
}
