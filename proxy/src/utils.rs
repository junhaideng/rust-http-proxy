use std::str;

use base64::decode as base64decode;

/// decode 从base64加密的数据中获取到用户名和密码
pub fn decode(string: &String) -> Result<(String, String), &str> {
    let tmp = &match base64decode(string) {
        Ok(res) => res,
        Err(_) => return Err("decode authorization failed"),
    };
    let auth: Vec<&str> = str::from_utf8(tmp)
        .expect("convert authorization data failed")
        .splitn(2, ':')
        .collect();
    if auth.len() == 2 {
        return Ok((String::from(auth[0]), String::from(auth[1])));
    }

    Err("Decode authorization information failed")
}

#[test]
fn decode_test() {
    let username = "username".to_string();
    let passwd = "password".to_string();
    let encode_msg = "dXNlcm5hbWU6cGFzc3dvcmQ".to_string();

    let result = decode(&encode_msg);
    assert!(result.is_ok());
    let (u, p) = result.unwrap();
    println!("{}, {}", u, p);
    assert_eq!(u, username);
    assert!(u.eq(&username) && p.eq(&passwd));
}
