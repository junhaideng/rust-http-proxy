//! 请求路径过滤

use super::FilterStatus;
use crate::{http, Config};

pub fn filter_request_path(config: &Config, request: &http::Request) -> FilterStatus {
    let path = &config.deny.request.line.path;
    if path.contains(&request.path) {
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}
