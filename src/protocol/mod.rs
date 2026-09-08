mod frame;
mod handshake;
mod proxy;

use crate::config::Messages;
use std::{
    io::{self, Cursor, Read},
    net::TcpStream,
    time::Duration,
};

const SOCKET_TIMEOUT: Duration = Duration::from_secs(2);

/**
 * Serve ping requests and reject login attempts
 */
pub fn handle_connection(stream: TcpStream, messages: &Messages) -> io::Result<()> {
    stream.set_read_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_nodelay(true)?;
    let mut stream = proxy::Stream::new(stream, messages.proxy_protocol_v2)?;

    match handshake::state(&frame::read_frame(&mut stream)?)? {
        1 => send_status(&mut stream, messages),
        2 => frame::send_text(&mut stream, 0, &messages.disconnect_json),
        _ => Err(frame::invalid("unsupported handshake state")),
    }
}

fn send_status(stream: &mut proxy::Stream, messages: &Messages) -> io::Result<()> {
    let request = frame::read_frame(stream)?;
    let mut request = Cursor::new(request);
    if frame::read_var_int(&mut request)? != 0
        || request.position() != request.get_ref().len() as u64
    {
        return Err(frame::invalid("expected status request"));
    }

    frame::send_text(
        stream,
        0,
        &format!("{}0{}", messages.status_prefix, messages.status_suffix),
    )?;

    let ping = match frame::read_frame(stream) {
        Ok(ping) => ping,
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::UnexpectedEof
            ) =>
        {
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    let mut ping = Cursor::new(ping);
    if frame::read_var_int(&mut ping)? != 1 || ping.get_ref().len() - ping.position() as usize != 8
    {
        return Ok(());
    }
    let mut token = [0; 8];
    ping.read_exact(&mut token)?;
    frame::send_packet(stream, 1, &token)
}
