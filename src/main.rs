use clap::{Parser, Subcommand};
use lunatic::{net::TcpListener, sleep, spawn_link, Mailbox};
use redis_lunatic::{client::Client, server::Server};
use std::{fmt::format, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Server {
        #[arg(short, long, default_value_t = 6379)]
        port: u16,
    },
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server { port } => {
            let server = spawn_link!(@task |port = port| {
                let listener = TcpListener::bind(format!("[::1]:{port}")).unwrap();
                let mut server = Server::new(listener);
                println!("Listening on port: {}", port);
                server.run();
            });
            let _ = server.result();
        }
    }
}
