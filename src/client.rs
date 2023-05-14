use crate::{connection::Connection, frame::Frame};
use bytes::Bytes;
use lunatic::net::TcpStream;
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::IpAddr,
};

#[derive(Debug)]
pub struct Client {
    connection: Connection<BufReader<TcpStream>, BufWriter<TcpStream>>,
}

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Ping,
    Set(String, Bytes),
    Get(String),
}

impl From<Command> for Frame {
    fn from(value: Command) -> Self {
        match value {
            Command::Ping => Frame::Array(vec![Frame::Bulk("PING".into())]),
            Command::Set(key, value) => Frame::Array(vec![
                Frame::Bulk("SET".into()),
                Frame::Bulk(key.into()),
                Frame::Bulk(value),
            ]),
            Command::Get(key) => {
                Frame::Array(vec![Frame::Bulk("GET".into()), Frame::Bulk(key.into())])
            }
        }
    }
}

impl Client {
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let writer = BufWriter::new(stream.clone());
        let reader = BufReader::new(stream);
        let connection = Connection::new(reader, writer);
        Ok(Self { connection })
    }

    pub fn ping(&mut self) -> Result<Frame, ()> {
        match self.send_command(Command::Ping) {
            Ok(Frame::Error(_)) => Err(()),
            Ok(frame) => Ok(frame),
            Err(_) => Err(()),
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Frame, ()> {
        match self.send_command(Command::Get(key.into())) {
            Ok(Frame::Error(_)) => Err(()),
            Ok(frame) => Ok(frame),
            Err(_) => Err(()),
        }
    }

    pub fn set(&mut self, key: &str, value: Bytes) -> Result<(), ()> {
        match self.send_command(Command::Set(key.into(), value)) {
            Ok(Frame::Error(_)) => Err(()),
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn send_command(&mut self, command: Command) -> Result<Frame, ()> {
        self.connection.send_frame(command.into())?;
        self.connection.receive_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use crate::{connection::Connection, frame::Frame};
    use lunatic::net::TcpStream;
    use pretty_assertions::assert_eq;
    use std::io::{BufReader, BufWriter};

    #[lunatic::test]
    fn ping() {
        let mut client = Client::connect("[::1]:6379").unwrap();
        let pong = client.ping().unwrap();
        assert_eq!(pong, Frame::Simple("PONG".to_string()))
    }

    #[lunatic::test]
    fn set() {
        let mut client = Client::connect("[::1]:6379").unwrap();
        let response = client.set("name", "unworthyEnzyme".into());
        assert!(response.is_ok());
    }

    #[lunatic::test]
    fn get() {
        let mut client = Client::connect("[::1]:6379").unwrap();
        client.set("name", "unworthyEnzyme".into()).unwrap();
        let response = client.get("name").unwrap();
        assert_eq!(response, Frame::Bulk("unworthyEnzyme".into()));
    }
}
