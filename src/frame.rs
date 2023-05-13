use bytes::{BufMut, Bytes, BytesMut};
use std::{
    io::{self, BufRead},
    vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i32),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn encode(&self) -> Bytes {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into(),
            Frame::Error(e) => format!("-{}\r\n", e).into(),
            Frame::Integer(i) => format!(":{}\r\n", i).into(),
            Frame::Bulk(b) => {
                let mut buffer = BytesMut::with_capacity(b.len());
                buffer.put_u8(b'$');
                buffer.put(b.len().to_string().as_bytes());
                buffer.put(&b"\r\n"[..]);
                buffer.put(&b[..]);
                buffer.put(&b"\r\n"[..]);
                buffer.into()
            }
            Frame::Null => "$-1\r\n".into(),
            Frame::Array(a) => {
                let mut buffer = BytesMut::with_capacity(a.len() * 1024);
                buffer.put_u8(b'*');
                buffer.put(a.len().to_string().as_bytes());
                buffer.put(&b"\r\n"[..]);
                for frame in a {
                    buffer.put(&frame.encode()[..]);
                }
                buffer.into()
            }
        }
    }

    pub fn decode<T: BufRead>(reader: &mut T) -> Result<Frame, DecodingError> {
        let mut buffer = vec![0; 1];
        reader.read_exact(&mut buffer)?;
        match buffer[0] {
            b'+' => {
                let mut buffer = String::with_capacity(1024);
                reader.read_line(&mut buffer)?;
                Ok(Frame::Simple(buffer.trim_end_matches("\r\n").to_string()))
            }
            b'-' => {
                let mut buffer = String::with_capacity(1024);
                reader.read_line(&mut buffer)?;
                Ok(Frame::Error(buffer.trim_end_matches("\r\n").to_string()))
            }
            b':' => {
                let num = Frame::decode_integer(reader);
                match num {
                    Ok(num) => Ok(Frame::Integer(num)),
                    Err(_) => Err(DecodingError::InvalidFormat),
                }
            }
            b'$' => {
                let length = Frame::decode_integer(reader)?;
                if length == -1 {
                    return Ok(Frame::Null);
                }
                let mut buffer = vec![0; length as usize + 2];
                reader.read_exact(&mut buffer)?;
                println!("{}", buffer.capacity());
                Ok(Frame::Bulk(Bytes::copy_from_slice(
                    &buffer[..buffer.len() - 2],
                )))
            }
            b'*' => {
                let length = Frame::decode_integer(reader)?;
                if length < 0 {
                    return Err(DecodingError::InvalidFormat);
                }
                let length = length as usize;
                let mut frames = Vec::with_capacity(length);
                for _ in 0..length {
                    frames.push(Frame::decode(reader)?);
                }
                Ok(Frame::Array(frames))
            }
            _ => Err(DecodingError::InvalidFormat),
        }
    }

    fn decode_integer<T: BufRead>(reader: &mut T) -> Result<i32, DecodingError> {
        let mut buffer = String::with_capacity(1024);
        reader.read_line(&mut buffer)?;
        let line = buffer.trim_end_matches("\r\n");
        let num = line.parse::<i32>();
        match num {
            Ok(num) => Ok(num),
            Err(_) => Err(DecodingError::InvalidFormat),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DecodingError {
    Incomplete,
    InvalidFormat,
    EOF,
    Other,
}

impl From<io::Error> for DecodingError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => Self::InvalidFormat,
            io::ErrorKind::UnexpectedEof
            | io::ErrorKind::Interrupted
            | io::ErrorKind::TimedOut
            | io::ErrorKind::BrokenPipe => Self::Incomplete,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Frame;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    #[lunatic::test]
    fn encodes_simple_string() {
        let frame = Frame::Simple("hello".to_string());
        assert_eq!(frame.encode(), ("+hello\r\n"))
    }

    #[lunatic::test]
    fn encodes_integer() {
        let frame = Frame::Integer(42);
        assert_eq!(frame.encode(), (":42\r\n"))
    }

    #[lunatic::test]
    fn encodes_error() {
        let frame = Frame::Error("error".to_string());
        assert_eq!(frame.encode(), ("-error\r\n"))
    }

    #[lunatic::test]
    fn encodes_null() {
        let frame = Frame::Null;
        assert_eq!(frame.encode(), ("$-1\r\n"))
    }

    #[lunatic::test]
    fn encodes_bulk() {
        let frame = Frame::Bulk("hello".into());
        assert_eq!(frame.encode(), ("$5\r\nhello\r\n".to_string()))
    }

    #[lunatic::test]
    fn encodes_array() {
        let frame = Frame::Array(vec![Frame::Bulk("hello".into()), Frame::Integer(42)]);
        assert_eq!(frame.encode(), ("*2\r\n$5\r\nhello\r\n:42\r\n"));
    }

    #[lunatic::test]
    fn decodes_simple_string() {
        let frame = Frame::Simple("hello".to_string());
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame))
    }

    #[lunatic::test]
    fn decodes_error() {
        let frame = Frame::Error("error".to_string());
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame))
    }

    #[lunatic::test]
    fn decodes_integer() {
        let frame = Frame::Integer(42);
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame));
    }

    #[lunatic::test]
    fn decodes_bulk() {
        let frame = Frame::Bulk("hello".into());
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame))
    }

    #[lunatic::test]
    fn decode_null() {
        let frame = Frame::Null;
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame));
    }

    #[lunatic::test]
    fn decodes_array() {
        let frame = Frame::Array(vec![Frame::Bulk("GET".into()), Frame::Bulk("key".into())]);
        let mut reader = Cursor::new(frame.encode());
        let decoded = Frame::decode(&mut reader);
        assert_eq!(decoded, Ok(frame))
    }
}
