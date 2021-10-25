//! 请求方法过滤

use crate::{http, Config};

use super::FilterStatus;

pub fn filter_request_method(config: &Config, request: &http::Request) -> FilterStatus {
    let methods = &config.deny.request.line.methods;
    let req_method = request.method.to_string();

    if methods.contains(&req_method) {
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}

#[test]
fn filter_request_method_test() {
    let mut config = Config::default();
    let mut request = http::Request::default();

    config.deny.request.line.methods.push("POST".to_string());
    assert_eq!(
        filter_request_method(&config, &request),
        FilterStatus::Forward
    );

    request.method = http::Method::POST;
    assert_eq!(
        filter_request_method(&config, &request),
        FilterStatus::Reject
    );
}
