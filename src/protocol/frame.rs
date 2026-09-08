use std::io::{self, Read, Write};

pub fn read_frame(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let length =
        usize::try_from(read_var_int(stream)?).map_err(|_| invalid("frame_len is negative"))?;
    if length == 0 || length > 4096 {
        return Err(invalid("frame_len out of range"));
    }
    let mut frame = vec![0; length];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

pub fn read_var_int(input: &mut impl Read) -> io::Result<i32> {
    let mut value = 0i32;
    for shift in 0..5 {
        let mut byte = [0];
        input.read_exact(&mut byte)?;
        value |= i32::from(byte[0] & 0x7f) << (shift * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(invalid("var_int is too large"))
}

pub fn send_text(stream: &mut impl Write, id: i32, text: &str) -> io::Result<()> {
    let mut payload = Vec::with_capacity(text.len() + 5);
    write_var_int(&mut payload, text.len() as i32);
    payload.extend_from_slice(text.as_bytes());
    send_packet(stream, id, &payload)
}

pub fn send_packet(stream: &mut impl Write, id: i32, payload: &[u8]) -> io::Result<()> {
    let mut body = Vec::with_capacity(payload.len() + 5);
    write_var_int(&mut body, id);
    body.extend_from_slice(payload);

    let mut frame = Vec::with_capacity(body.len() + 5);
    write_var_int(&mut frame, body.len() as i32);
    frame.extend_from_slice(&body);
    stream.write_all(&frame)
}

fn write_var_int(output: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            return;
        }
    }
}

pub fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
