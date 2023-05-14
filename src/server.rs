use self::db::{DBMessages, DBRequests, DB};
use crate::connection::Connection;
use crate::{command, frame::Frame};
use lunatic::{
    ap::ProcessRef,
    net::{TcpListener, TcpStream},
    AbstractProcess, Mailbox, Process,
};
use std::{
    collections::HashMap,
    io::{BufReader, BufWriter},
};

mod db {
    use bytes::Bytes;
    use lunatic::{abstract_process, ap::Config, Tag};
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub struct DB {
        entries: HashMap<String, Bytes>,
    }

    #[abstract_process(visibility = pub)]
    impl DB {
        #[init]
        fn init(_: Config<Self>, start: HashMap<String, Bytes>) -> Result<Self, ()> {
            Ok(Self { entries: start })
        }

        #[handle_message]
        fn set(&mut self, key: String, value: Bytes) {
            self.entries.insert(key, value);
        }

        #[handle_request]
        fn get(&self, key: String) -> Option<Bytes> {
            self.entries.get(&key).cloned()
        }

        #[terminate]
        fn terminate(self) {
            println!("Shutdown process");
        }

        #[handle_link_death]
        fn handle_link_death(&self, _tag: Tag) {
            println!("Link trapped");
        }
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub fn run(&mut self) {
        let db = DB::link().start(HashMap::new()).unwrap();
        loop {
            let (stream, _) = self.listener.accept().unwrap();
            Process::spawn((stream, db), handle);
        }
    }
}

fn handle((stream, db): (TcpStream, ProcessRef<DB>), _: Mailbox<()>) {
    let reader = BufReader::new(stream.clone());
    let writer = BufWriter::new(stream);
    let mut connection = Connection::new(reader, writer);
    loop {
        let frame = connection.receive_frame().unwrap();
        let command: command::Command = frame.try_into().unwrap();
        match command {
            command::Command::Ping => connection.send_frame(Frame::Simple("PONG".into())).unwrap(),
            command::Command::Set(key, value) => {
                db.set(key, value);
                connection.send_frame(Frame::Simple("OK".into())).unwrap();
            }
            command::Command::Get(key) => {
                let value = db.get(key);
                match value {
                    Some(value) => connection.send_frame(Frame::Bulk(value)).unwrap(),
                    None => connection.send_frame(Frame::Null).unwrap(),
                };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    #[lunatic::test]
    fn ping() {}
}
