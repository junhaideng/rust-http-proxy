use std::net::TcpStream;

/// 消息传递
///
/// 程序退出的时候会发送 Terminal 消息，线程池中的线程一个一个进行关闭
pub enum Message {
    NewStream(TcpStream),
    Terminate,
}
