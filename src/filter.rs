use proxy::filter::FilterStatus;
use proxy::http;
use proxy::Config;

/// 过滤请求方法
pub fn filter_method(config: &Config, request: &http::Request) -> FilterStatus {
    let methods = &config.deny.request.line.methods;
    let req_method = request.method.to_string();

    if methods.contains(&req_method) {
        return FilterStatus::Reject;
    }
    FilterStatus::Forward
}
