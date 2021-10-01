use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

use coach::entry;
use coach::files;

// A typical entry made by hand right now is around 1-2K
const MAX_ENTRY_SIZE_BYTES: usize = 8 * 1024;

const DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month repr:numerical]-[day]");

// TODO this validator for obs names is WRONG, I can say "Thing:thing:thing" "value"...
fn no_newline_validator(val: String) -> Result<(), String> {
    match entry::as_no_newlines(&val) {
        Some(_) => Ok(()),
        None => Err(String::from("argument can't contain newlines")),
    }
}

fn observation_name_validator(val: String) -> Result<(), String> {
    let nn = match entry::as_no_newlines(&val) {
        Some(nn) => nn,
        None => return Err(String::from("observation names can't contain newlines")),
    };

    if nn.to_string().contains(':') {
        Err(String::from(
            "observation names can't contain the colon character",
        ))
    } else {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new("coach")
        .about("a journal and project manager")
        .subcommand(
            SubCommand::with_name("today")
                .about("creates a new journal file in the current working directory"),
        )
        .subcommand(
            SubCommand::with_name("observe")
                .about("adds a key/value observation to the journal")
                .arg(
                    Arg::with_name("NAME")
                        .required(true)
                        .validator(observation_name_validator)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .required(true)
                        .validator(no_newline_validator)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("cat")
                .about("writes the contents of a journal entry to standard out"),
        );
    let matches = app.clone().get_matches();

    let moment = SystemTime::now();
    let dt: OffsetDateTime = moment.into();
    let dt_formatted = dt.format(&DATE_FORMAT)?;
    let dt_label = entry::promise_no_newlines(&dt_formatted);

    match matches.subcommand_name() {
        Some("today") => {
            let entry = entry::Entry {
                label: dt_label,
                ..entry::Entry::default()
            };
            let mut out = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(entry.label.to_string())?;
            out.write_all(entry.to_string().as_bytes())?;
            out.sync_all()?;
        }
        Some("cat") => {
            with_entry_from_file(&dt_label.to_string(), |entry| {
                print!("{}", entry);
                Ok(())
            })?;
        }
        Some("observe") => {
            let name_str = matches.value_of("NAME").unwrap();
            let value_str = matches.value_of("VALUE").unwrap();

            let name = entry::promise_no_newlines(name_str);
            let value = entry::promise_no_newlines(value_str);

            edit_file(&dt_label.to_string(), |_buf, entry| {
                let obs = entry::Observation { name, value };
                println!("WOULD HAVE OBSERVED {}", obs);
                // AS FAR AS I CAN TELL, we can't really see what entry's lifetime is from here
                // (we get a reference to "some lifetime" but maybe it's longer than the
                //  enclosing scope.)
                //
                // We need a way to promise that the lifetime of entry is no longer than
                // the lifetime of (say) dt_label.
                //
                // Could it be moved into here instead?
                // entry.observations.push(obs);
                Ok(())
            })?;
        }
        Some(_) | None => {
            let _ = app.print_long_help();
            println!();
        }
    };

    Ok(())

    /* TODO CLEANUP
    let moment = SystemTime::now();
    let dt: OffsetDateTime = moment.into();
    let dt_label = dt.format(&DATE_FORMAT)?;

    let mut today = File::create(&dt_label)?;

    let sample = entry::Entry {
        label: entry::promise_no_newlines(&dt_label),
        observations: vec![entry::Observation {
            name: entry::promise_no_newlines("example"),
            value: entry::promise_no_newlines("this is an example entry"),
        }],
        tasks: vec![
            entry::Task::Done(entry::promise_no_newlines("Write an example entry")),
            entry::Task::Todo(entry::promise_no_newlines("Read an entry from a file")),
        ],
        events: vec![entry::Event {
            when: dt,
            text: entry::promise_no_newlines("created a cool new file"),
        }],
        notes: vec![entry::promise_nonempty_note(
            "Notes go here, after observations and tasks",
        )],
    };

    today.write_all(b"coach1\n")?;
    today.write_all(sample.to_string().as_bytes())?;
    today.sync_all()?;
    Ok(())
    ***/
}

fn with_entry_from_file<F, T>(filename: &str, f: F) -> Result<T, Box<dyn Error>>
where
    F: FnOnce(&mut entry::Entry) -> Result<T, Box<dyn Error>>,
{
    let mut buf: Vec<u8> = Vec::new();
    let text = files::read_bounded_str_from_file(&mut buf, filename, MAX_ENTRY_SIZE_BYTES)?;
    let mut entry = entry::Entry::default();
    let _ = entry::parse(text, &mut entry)?;

    f(&mut entry)
}

fn edit_file<F>(filename: &str, f: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(&String, &mut entry::Entry) -> Result<(), Box<dyn Error>>,
{
    with_entry_from_file(filename, |mut entry| {
        let buf = String::new();
        let _ = f(&buf, &mut entry)?;
        println!("TODO WOULD HAVE EDITED:\n{}", entry);
        Ok(())
    })
}
