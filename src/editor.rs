use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::process::Command;
use tempfile::NamedTempFile;

// TODO might be nice to write a prompt to the file?
pub fn edit_prompt() -> Result<String, io::Error> {
    let (mut file, path) = NamedTempFile::new()?.into_parts();
    launch_editor(path.to_str().unwrap())?;
    let mut ret = String::new();
    file.read_to_string(&mut ret)?;
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
