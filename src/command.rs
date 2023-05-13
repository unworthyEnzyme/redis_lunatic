use crate::frame::Frame;
use bytes::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
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

impl TryFrom<Frame> for Command {
    type Error = ();

    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let Frame::Array(frames) = value else {
            return  Err(());
        };
        let Some(name) = frames.get(0) else {
            return Err(());
        };
        let Frame::Bulk(name) = name else {
            return  Err(());
        };
        if name == &Bytes::from("PING") {
            Ok(Command::Ping)
        } else if name == &Bytes::from("GET") {
            let Some(Frame::Bulk(key)) = frames.get(1) else {
                return Err(());
            };
            let Ok(key) = String::from_utf8(key.to_vec()) else {
                return Err(());
            };
            return Ok(Command::Get(key));
        } else if name == &Bytes::from("SET") {
            let (Some(Frame::Bulk(key)), Some(Frame::Bulk(value))) = (frames.get(1), frames.get(2)) else {
                return Err(());
            };
            let Ok(key) = String::from_utf8(key.to_vec()) else {
                return Err(());
            };
            return Ok(Command::Set(key, value.clone()));
        } else {
            return Err(());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{command::Command, frame::Frame};
    use pretty_assertions::assert_eq;

    #[test]
    fn ping_to_frame() {
        let frame: Frame = Command::Ping.into();
        assert_eq!(frame, Frame::Array(vec![Frame::Bulk("PING".into())]));
    }

    #[test]
    fn get_to_frame() {
        let frame: Frame = Command::Get("name".into()).into();
        assert_eq!(
            frame,
            Frame::Array(vec![Frame::Bulk("GET".into()), Frame::Bulk("name".into())])
        );
    }

    #[test]
    fn set_to_frame() {
        let frame: Frame = Command::Set("name".into(), "unworthyEnzyme".into()).into();
        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::Bulk("SET".into()),
                Frame::Bulk("name".into()),
                Frame::Bulk("unworthyEnzyme".into())
            ])
        );
    }
}
