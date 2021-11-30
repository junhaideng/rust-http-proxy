use log::error;
use std::io::Write;
use std::net::{Shutdown, TcpStream};

static HTTP_AUTH: &[u8] = "HTTP/1.1 401 Unauthorized\r\nConnection: close\r\n\r\n".as_bytes();
static HTTP_FORBIDDEN: &[u8] = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n".as_bytes();
static HTTP_PROXY_AUTH: &[u8] =
    "HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic\r\n\r\n".as_bytes();
static HTTP_NOT_SUPPORT: &[u8] = "HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 31\r\n\r\nProxy do not support https Now".as_bytes();
static HTTP_STATUS_OK: &[u8] = "HTTP/1.1 200 OK\r\nProxy-Connection: keep-alive\r\n\r\n".as_bytes();

pub fn unauthorized(stream: &mut TcpStream) {
    if let Err(err) = stream.write(HTTP_AUTH) {
        error!("write stream failed: {}", err);
    }
    if let Err(err) = stream.shutdown(Shutdown::Both) {
        error!("shutdown stream failed: {}", err);
    }
}

pub fn forbidden(stream: &mut TcpStream) {
    if let Err(err) = stream.write(HTTP_FORBIDDEN) {
        error!("write stream failed: {}", err);
    }
    if let Err(err) = stream.shutdown(Shutdown::Both) {
        error!("shutdown stream failed: {}", err);
    }
}

pub fn proxy_auth(stream: &mut TcpStream) {
    if let Err(err) = stream.write(HTTP_PROXY_AUTH) {
        error!("write stream failed: {}", err);
    }
    if let Err(err) = stream.shutdown(Shutdown::Both) {
        error!("shutdown stream failed: {}", err);
    }
}

pub fn not_support_https(stream: &mut TcpStream) {
    if let Err(err) = stream.write(HTTP_NOT_SUPPORT) {
        error!("write stream failed: {}", err);
    }
    if let Err(err) = stream.shutdown(Shutdown::Both) {
        error!("shutdown stream failed: {}", err);
    }
}

pub fn http_status_ok(stream: &mut TcpStream) {
    if let Err(err) = stream.write(HTTP_STATUS_OK) {
        error!("write stream failed: {}", err);
    }
    if let Err(err) = stream.flush() {
        error!("flush stream failed: {}", err);
    }
}
