use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read};
use std::str;

use crate::entry;

// "A Christmas Carol" in plain text on project Gutenberg is around 170 Kb
// A typical entry made by hand right now is around 1-2K
const MAX_ENTRY_SIZE_BYTES: usize = 8 * 1024;

// Security - if you label your entry /boot/vmlinuz you're going to have a bad time.
// TODO pull File / Read out of this so you can test it please.
// TODO this should be two fns anyhow, so we can more easily field the parse error
pub fn read_entry_from_file<'a>(
    storage: &'a mut Vec<u8>,
    dest: &mut entry::Entry<'a>,
    filename: &str, // TODO filename should be a path
) -> Result<usize, io::Error> {
    storage.resize(MAX_ENTRY_SIZE_BYTES, 0);
    let mut entry_size = 0;
    let mut chunk = &mut storage[..];
    match File::open(filename) {
        Ok(mut f) => loop {
            match f.read(chunk) {
                Ok(0) => break,
                Ok(n) => {
                    entry_size += n;
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
            "entry file longer than maximum length allowed",
        ));
    }

    let text = match str::from_utf8(storage) {
        Ok(text) => text,
        Err(e) => {
            return Err(io::Error::new(ErrorKind::InvalidData, e));
        }
    };

    if let Err(e) = entry::parse(text, dest) {
        // TODO callers should be able to see parse errors as different
        // from io or encoding errors.
        return Err(io::Error::new(ErrorKind::InvalidData, e));
    }

    Ok(entry_size)
}
