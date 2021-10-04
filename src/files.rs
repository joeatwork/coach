use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read};
use std::str;

pub fn read_bounded_str_from_file<'a>(
    buf: &'a mut Vec<u8>,
    filename: &str,
    max_length: usize,
) -> Result<&'a str, io::Error> {
    buf.resize(max_length, 0);
    let mut chunk = &mut buf[..];
    let mut read_length: usize = 0;
    match File::open(filename) {
        Ok(mut f) => loop {
            match f.read(chunk) {
                Ok(0) => break,
                Ok(n) => {
                    read_length += n;
                    chunk = &mut chunk[n..];
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        },
        Err(read_err) => return Err(read_err),
    };

    if chunk.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "file is longer than maximum length allowed",
        ));
    }

    buf.truncate(read_length);
    let text = match str::from_utf8(buf) {
        Ok(text) => text,
        Err(e) => {
            return Err(io::Error::new(ErrorKind::InvalidData, e));
        }
    };

    Ok(text)
}
