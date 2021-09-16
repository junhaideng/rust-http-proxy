use crate::config::Config;
use hyper::HeaderMap;

// 进行过滤
pub fn filter(_config: &Config, header: &HeaderMap) -> bool {
    for (key, value) in header.iter() {
        println!("key: {}, value: {:?}", key, value);
    }

    false
}

#[test]
fn filter_test() {
    let map = HeaderMap::new();
    // map.insert("key", HeaderValue::from_static("hello"));
    filter(&Config::generate_default().unwrap(), &map);
}
