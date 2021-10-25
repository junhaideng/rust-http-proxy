//! 请求路径过滤

use crate::{http, Config};

use super::FilterStatus;

pub fn filter_request_path(config: &Config, request: &http::Request) -> FilterStatus {
    let path = &config.deny.request.line.path;
    if path.contains(&request.path) {
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}

#[test]
fn filter_request_path_test() {
    let mut config = Config::default();
    let mut request = http::Request::default();

    config.deny.request.line.path.push("/login".to_string());
    assert_eq!(
        filter_request_path(&config, &request),
        FilterStatus::Forward
    );
    request.path = "/login".to_string();
    assert_eq!(filter_request_path(&config, &request), FilterStatus::Reject);
}
