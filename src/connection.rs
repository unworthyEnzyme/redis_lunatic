use crate::frame::{self, Frame};
use std::io::{self, BufRead, BufWriter, Write};
use thiserror::Error;

#[derive(Debug)]
pub struct Connection<R, W> {
    reader: R,
    writer: W,
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Decoding error")]
    DecodingError(#[from] frame::DecodingError),
}

impl<R: BufRead, W: Write> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    pub fn send_frame(&mut self, frame: Frame) -> Result<(), ConnectionError> {
        let mut writer = BufWriter::new(&mut self.writer);
        writer.write_all(&frame.encode()[..])?;
        writer.flush()?;
        Ok(())
    }

    pub fn receive_frame(&mut self) -> Result<Frame, ConnectionError> {
        Ok(Frame::decode(&mut self.reader)?)
    }
}
