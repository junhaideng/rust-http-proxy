use clap::{App, Arg};
use proxy::Server;

fn main() {
    let app = App::new("rust proxy")
        .version("1.0.0")
        .about("a simple http proxy using rust")
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
                .default_value("10"),
        )
        .arg(
            Arg::with_name("transparent")
                .short("t")
                .long("transparent")
                .help("set transparent proxy [true, false]")
                .default_value("false"),
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
        Some(size) => match size.parse() {
            Ok(r) => r,
            Err(_) => {
                println!("pool_size must be an integer");
                return;
            }
        },
        None => 4,
    };

    let flag = match app.value_of("transparent") {
        Some(f) => match f {
            "true" => true,
            "false" => false,
            _ => {
                println!("非法的 transparent 参数: {}， 只能为 true 或者 false", f);
                return;
            }
        },
        None => false,
    };

    let mut s = match Server::new(&host, &port, size) {
        Ok(server) => server,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if flag {
        s.init_iptables();
    }

    if let Err(err) = s.run() {
        println!("{}", err)
    };
}
