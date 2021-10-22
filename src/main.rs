mod filter;
use proxy::Server;

fn main() {
    let mut s = Server::new("0.0.0.0", "8080", 4);
    s.add_request_filter(filter::filter_method);

    s.run().unwrap();
}
