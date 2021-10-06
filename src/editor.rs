use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io;
use std::process::Command;

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
