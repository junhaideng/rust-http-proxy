use clap::{App, Arg};
use proxy::Server;

fn main() {
    let app = App::new("rust proxy")
        .version("1.0")
        .about("simple http proxy")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .help("set server host")
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("proxy server port")
                .default_value("8080"),
        )
        .arg(
            Arg::with_name("pool_size")
                .short("s")
                .long("pool_size")
                .help("proxy server pool size")
                .default_value("4"),
        )
        .get_matches();

    let host = match app.value_of("host") {
        Some(host) => host.to_string(),
        None => "0.0.0.0".to_string(),
    };

    let port = match app.value_of("port") {
        Some(port) => port.to_string(),
        None => "8080".to_string(),
    };

    let size: usize = match app.value_of("pool_size") {
        Some(size) => size.parse().expect("pool_size must be an integer"),
        None => 4,
    };

    let mut s = Server::new(&host, &port, size);
    s.run().unwrap();
}
