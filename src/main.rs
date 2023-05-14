use std::{
    io::{BufReader, BufWriter},
    time::Duration,
};

use lunatic::{
    net::{TcpListener, TcpStream},
    sleep, spawn_link, Mailbox,
};
use redis_lunatic::{client::Client, connection::Connection, server::Server};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let server = spawn_link!(@task || {
        let listener = TcpListener::bind("localhost:3000").unwrap();
        let mut server = Server::new(listener);
        server.run();
    });
    let _client = spawn_link!(@task || {
        sleep(Duration::from_secs(1));
        let mut client = Client::connect("localhost:3000").unwrap();
        client.set("name", "unworthyEnyzme".into()).unwrap();
        let name = client.get("name").unwrap();
        println!("[client]: {:?}", name);
    });
    server.result();
}
