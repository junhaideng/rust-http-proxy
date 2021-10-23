use proxy::Server;

fn main() {
    let mut s = Server::new("0.0.0.0", "8080", 4);

    s.run().unwrap();
}
