use std::collections::HashMap;

/// 头部过滤器
///
/// 对头部的信息进行过滤，比如声明 key: value
/// 那么如果头部包含 key，并且对应的头部值**包含** value
/// 那么会被过滤掉
// 过滤头部，通用方法
pub fn filter_header(headers: &HashMap<String, String>, key: &str, value: &str) -> bool {
    match headers.get(key) {
        Some(v) => {
            if v.contains(value) {
                return true;
            }
        }
        None => {}
    }
    false
}

#[test]
fn filter_test() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer hello".to_string());
    headers.insert("Host".to_string(), "www.baidu.com".to_string());

    assert!(filter_header(&headers, "Content-Type", "json"));

    assert!(!filter_header(&headers, "Host", "www.google.com"));
}
