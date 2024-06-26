use std::time::Duration;

use clap::Parser;

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
}

fn main() {
    let args = Args::parse();
    if args.server && !args.client {
        server::run();
    }

    if args.client && !args.server {
        client::run();
    }

    if args.server && args.client {
        std::thread::spawn(|| server::run());
        std::thread::sleep(Duration::from_millis(1000));
        client::run();
    }
}
