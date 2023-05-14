use clap::{Parser, Subcommand};
use lunatic::{net::TcpListener, spawn_link, Mailbox};
use redis_lunatic::server::Server;

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
            server.result();
        }
    }
}
