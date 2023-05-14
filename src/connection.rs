use crate::frame::Frame;
use std::io::{BufRead, BufWriter, Write};

#[derive(Debug)]
pub struct Connection<R, W> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    pub fn send_frame(&mut self, frame: Frame) -> Result<(), ()> {
        let mut writer = BufWriter::new(&mut self.writer);
        if writer.write_all(&frame.encode()[..]).is_err() {
            return Err(());
        };
        if writer.flush().is_err() {
            return Err(());
        };
        Ok(())
    }

    pub fn receive_frame(&mut self) -> Result<Frame, ()> {
        let response = Frame::decode(&mut self.reader);
        let Ok(response) = response else {
            return Err(())
        };
        Ok(response)
    }
}
