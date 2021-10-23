use log::error;
use std::io::Write;
use std::net::{Shutdown, TcpStream};

static HTTP_AUTH: &[u8] = "HTTP/1.1 401 Unauthorized\r\nConnection: close\r\n\r\n".as_bytes();
static HTTP_FORBIDDEN: &[u8] = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n".as_bytes();
static HTTP_PROXY_AUTH: &[u8] =
    "HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic\r\n\r\n".as_bytes();

pub fn unauthorized(stream: &mut TcpStream) {
    match stream.write(HTTP_AUTH) {
        Ok(_) => {}
        Err(err) => {
            error!("write stream failed: {}", &err);
        }
    };
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => {}
        Err(err) => {
            error!("shutdown stream failed: {}", &err);
        }
    };
}

pub fn forbidden(stream: &mut TcpStream) {
    match stream.write(HTTP_FORBIDDEN) {
        Ok(_) => {}
        Err(err) => {
            error!("write stream failed: {}", &err);
        }
    };
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => {}
        Err(err) => {
            error!("shutdown stream failed: {}", &err);
        }
    };
}

pub fn proxy_auth(stream: &mut TcpStream) {
    match stream.write(HTTP_PROXY_AUTH) {
        Ok(_) => {}
        Err(err) => {
            error!("write stream failed: {}", &err);
        }
    };
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => {}
        Err(err) => {
            error!("shutdown stream failed: {}", &err);
        }
    };
}
