use proxy::Server;

fn main() {
    let mut s = Server::new("127.0.0.1", "8080", 4);
    s.run().unwrap();
}
