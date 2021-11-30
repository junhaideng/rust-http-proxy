//! 请求路径过滤

use crate::http;
use log::error;
use regex::Regex;

// 如果请求路径中包含在 paths 中，则应该被过滤
pub fn filter_request_path(paths: &Vec<String>, request: &http::Request) -> bool {
    for path in paths.iter() {
        let reg = match Regex::new(path) {
            Ok(e) => e,
            Err(e) => {
                error!("Regex error: {}", &e);
                return false;
            }
        };
        if reg.is_match(&request.path) {
            return true;
        }
    }
    false
}

#[test]
fn filter_request_path_test() {
    let mut path = Vec::new();
    let mut request = http::Request::default();

    path.push("/login".to_string());
    assert!(!filter_request_path(&path, &request));

    request.path = "/login".to_string();
    assert!(filter_request_path(&path, &request));

    path.push("/admin/.*".to_string());
    assert!(filter_request_path(&path, &request));
    request.path = "/admin/user_id".to_string();
    assert!(filter_request_path(&path, &request));

    request.path = "/admin_suffix".to_string();
    assert!(!filter_request_path(&path, &request));
}
