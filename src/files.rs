use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::io::{ErrorKind, Read};
use std::str;

use crate::entry;

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

pub fn new_entry_file(filename: &str, entry: &entry::Entry) -> Result<(), io::Error> {
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&filename)?;
    out.write_all(entry.to_string().as_bytes())?;
    out.sync_all()?;

    Ok(())
}

pub fn entry_from_file(filename: &str, max_size: usize) -> Result<entry::Entry, Box<dyn Error>> {
    let mut buf: Vec<u8> = Vec::new();
    let text = read_bounded_str_from_file(&mut buf, filename, max_size)?;
    match entry::parse(text) {
        Ok(e) => Ok(e),
        Err(e) => Err(Box::new(e)),
    }
}

// will *not* create a new file.
pub fn entry_to_file(filename: &str, entry: &entry::Entry) -> Result<(), io::Error> {
    let mut newfile = OpenOptions::new()
        .write(true)
        .create_new(false)
        .open(&filename)?;
    newfile.write_all(entry.to_string().as_bytes())?;
    newfile.sync_all()?;

    Ok(())
}
