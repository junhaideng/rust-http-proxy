//! 请求路径过滤

use super::FilterStatus;
use crate::{http, Config};
use log::error;
use regex::Regex;

pub fn filter_request_path(config: &Config, request: &http::Request) -> FilterStatus {
    let paths = &config.deny.request.line.path;
    for path in paths.iter() {
        let reg = match Regex::new(path) {
            Ok(e) => e,
            Err(e) => {
                error!("Regex error: {}", &e);
                return FilterStatus::Reject;
            }
        };
        if reg.is_match(&request.path) {
            return FilterStatus::Reject;
        }
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

    config.deny.request.line.path.push("/admin/.*".to_string());
    assert_eq!(filter_request_path(&config, &request), FilterStatus::Reject);
    request.path = "/admin/user_id".to_string();
    assert_eq!(filter_request_path(&config, &request), FilterStatus::Reject);

    request.path = "/admin_suffix".to_string();
    assert_eq!(
        filter_request_path(&config, &request),
        FilterStatus::Forward
    );
}
