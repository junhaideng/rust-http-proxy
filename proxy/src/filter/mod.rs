use crate::config::Config;
use crate::http;

pub mod header;
pub mod method;
pub mod path;

/// 过滤状态
///
/// 被过滤还是进行转发代理
#[derive(Debug, PartialEq, Eq)]
pub enum FilterStatus {
    Forward,
    Reject,
}

/// 对客户端的请求进行过滤
pub type FilterRequest = fn(config: &Config, request: &http::Request) -> FilterStatus;

/// 对服务端返回的响应进行过滤
pub type FilterResponse = fn(config: &Config, response: &http::Response) -> FilterStatus;
