use crate::config::Config;
use crate::http;

lazy_static! {
    static ref CFG: Config = Config::parse("config.yml").unwrap();
}

// 进行过滤
pub fn filter_response(response: &http::Response) -> bool {
    // check content-type
    let headers = &response.headers;
    let content_type = headers.get("Content-Type").unwrap();

    //
    for typ in &CFG.content_type {
        if content_type.contains(typ) {
            return true;
        }
    }

    false
}

#[test]
fn filter_test() {}
