use std::env;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::process::Command;
use tempfile::NamedTempFile;

// TODO might be nice to write a prompt to the file?
pub fn edit_prompt() -> Result<String, io::Error> {
    let tf = NamedTempFile::new().unwrap();
    let path = tf.into_temp_path();
    launch_editor(path.to_str().unwrap())?;
    let mut ret = String::new();

    // Can't use tf.reopen(), it (appears) that vi
    // on my mac doesn't actually edit the file, just
    // copies and renames.
    let mut inf = File::open(path)?;
    inf.read_to_string(&mut ret)?;
    Ok(ret)
}

// $EDITOR support is minimal - EDITOR isn't run through a shell,
// so cool (and common!) tricks like EDITOR='vim -e' will break.
pub fn launch_editor(filename: &str) -> Result<(), io::Error> {
    let vi = OsString::from("vi");
    let editor = env::var_os("EDITOR").unwrap_or(vi);

    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    let tty_in = tty.try_clone()?;
    let mut editor = Command::new(editor)
        .arg(filename)
        .stdin(tty_in)
        .stdout(tty)
        .spawn()?;
    editor.wait().map(|_| ())
}
