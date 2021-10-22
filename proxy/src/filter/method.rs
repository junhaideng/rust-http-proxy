//! 请求方法过滤

use crate::{Config, http};
use super::FilterStatus;

pub fn filter_request_method(config: &Config, request: &http::Request) -> FilterStatus {
    let methods = &config.deny.request.line.methods;
    let req_method = request.method.to_string();

    if methods.contains(&req_method) {
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}
