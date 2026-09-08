use super::frame::{invalid, read_var_int};
use std::{
    io::{self, Cursor, Read},
    str,
};

/**
 * Validate a handshake, and return its next state
 * - status: 1
 * - login: 2
 */
pub fn state(frame: &[u8]) -> io::Result<i32> {
    let mut input = Cursor::new(frame);
    if read_var_int(&mut input)? != 0 {
        return Err(invalid("expected handshake"));
    }
    let _client_protocol = read_var_int(&mut input)?;
    let host_len =
        usize::try_from(read_var_int(&mut input)?).map_err(|_| invalid("negative host length"))?;
    if host_len > 255 {
        return Err(invalid("invalid handshake host"));
    }
    let mut host = [0; 255];
    input.read_exact(&mut host[..host_len])?;
    str::from_utf8(&host[..host_len]).map_err(|_| invalid("invalid host utf-8"))?;

    let mut port = [0; 2];
    input.read_exact(&mut port)?;
    let state = read_var_int(&mut input)?;
    if input.position() != frame.len() as u64 {
        return Err(invalid("trailing handshake data"));
    }
    Ok(state)
}
