use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

/**
 * PROXY Protocol v2 is an application-layer prefix
 * appears as the binary signature `\r\n\r\n\0\r\nQUIT\n`
 */
const SIGNATURE: [u8; 12] = *b"\r\n\r\n\0\r\nQUIT\n";
const MAX_DATA: usize = 4096;

pub struct Stream {
    stream: TcpStream,
    prefix: [u8; 12],
    position: usize,
    end: usize,
}

impl Stream {
    pub fn new(mut stream: TcpStream, enabled: bool) -> io::Result<Self> {
        let mut prefix = [0; 12];
        if !enabled {
            return Ok(Self {
                stream,
                prefix,
                position: 0,
                end: 0,
            });
        }
        for end in 0..prefix.len() {
            stream.read_exact(&mut prefix[end..=end])?;
            if prefix[end] != SIGNATURE[end] {
                return Ok(Self {
                    stream,
                    prefix,
                    position: 0,
                    end: end + 1,
                });
            }
        }

        let mut header = [0; 4];
        stream.read_exact(&mut header)?;
        if header[0] >> 4 != 2 {
            return Err(invalid("unsupported proxy protocol version"));
        }
        let length = usize::from(u16::from_be_bytes([header[2], header[3]]));
        if length > MAX_DATA {
            return Err(invalid("proxy protocol header too large"));
        }
        let mut data = [0; MAX_DATA];
        stream.read_exact(&mut data[..length])?;

        Ok(Self {
            stream,
            prefix,
            position: 0,
            end: 0,
        })
    }
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.position < self.end {
            let length = buffer.len().min(self.end - self.position);
            buffer[..length].copy_from_slice(&self.prefix[self.position..self.position + length]);
            self.position += length;
            return Ok(length);
        }
        self.stream.read(buffer)
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.stream.write(buffer)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
