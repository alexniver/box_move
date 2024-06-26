use std::{net::IpAddr, time::Duration};

use clap::Parser;
use consts::SERVER_HOST;

mod client;
mod consts;
mod protocol;
mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    server: bool,
    #[arg(short, long)]
    client: bool,
    #[arg(long)]
    host: Option<String>,
}

fn main() {
    let args = Args::parse();

    let host: IpAddr = match args.host {
        Some(host) => host.parse().unwrap(),
        None => SERVER_HOST.parse().unwrap(),
    };

    if args.server && !args.client {
        server::run();
    }

    if args.client && !args.server {
        client::run(host);
    }

    if args.server && args.client {
        std::thread::spawn(|| server::run());
        std::thread::sleep(Duration::from_millis(1000));
        client::run(host);
    }
}
