use std::io::{Error, Read, Write};
use std::net::TcpStream;

use crate::{Frame, OpCode};

pub struct WebSocket {
    stream: TcpStream,
    read_buf: Vec<u8>,
}

impl WebSocket {
    #[must_use]
    pub fn new(stream: TcpStream) -> Self {
        let _ = stream.set_read_timeout(None);
        let _ = stream.set_write_timeout(None);
        Self {
            stream,
            read_buf: Vec::with_capacity(4096),
        }
    }

    /// # Errors
    /// Returns error if socket read fails
    pub fn recv_frame(&mut self) -> Result<Option<Frame>, Error> {
        let mut tmp = [0_u8; 4096];
        loop {
            if let Some((frame, consumed)) = Frame::decode(&self.read_buf) {
                self.read_buf.drain(..consumed);
                return Ok(Some(frame));
            }

            let n = self.stream.read(&mut tmp)?;
            if n == 0 {
                return Ok(None);
            }
            self.read_buf.extend_from_slice(&tmp[..n]);
        }
    }

    /// # Errors
    /// Returns error if socket write fails
    pub fn send_frame(&mut self, frame: &Frame) -> Result<(), Error> {
        let raw = frame.encode();
        self.stream.write_all(&raw)?;
        self.stream.flush()
    }

    /// # Errors
    /// Returns error if sending frame fails
    pub fn send_text(&mut self, text: &str) -> Result<(), Error> {
        let frame = Frame::new(true, OpCode::Text, text.as_bytes().to_vec());
        self.send_frame(&frame)
    }

    /// # Errors
    /// Returns error if sending frame fails
    pub fn send_binary(&mut self, data: &[u8]) -> Result<(), Error> {
        let frame = Frame::new(true, OpCode::Binary, data.to_vec());
        self.send_frame(&frame)
    }

    /// # Errors
    /// Returns error if sending frame fails
    pub fn send_ping(&mut self, data: &[u8]) -> Result<(), Error> {
        let frame = Frame::new(true, OpCode::Ping, data.to_vec());
        self.send_frame(&frame)
    }

    /// # Errors
    /// Returns error if sending frame fails
    pub fn send_pong(&mut self, data: &[u8]) -> Result<(), Error> {
        let frame = Frame::new(true, OpCode::Pong, data.to_vec());
        self.send_frame(&frame)
    }

    /// # Errors
    /// Returns error if sending close frame fails
    pub fn close(&mut self, code: u16, reason: &str) -> Result<(), Error> {
        let raw = Frame::encode_close(code, reason);
        self.stream.write_all(&raw)?;
        self.stream.flush()
    }
}
