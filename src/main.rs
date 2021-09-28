use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

use coach::entry;
use coach::files;

const DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month repr:numerical]-[day]");

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
                .arg(Arg::with_name("NAME").required(true).index(1))
                .arg(Arg::with_name("VALUE").required(true).index(2)),
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
        Some("today") => println!("WOULDA RUN TODAY"),
        Some("observe") => println!("WOULDA RUN OBSERVE"),
        Some("cat") => {
            let mut entry = entry::Entry::default();
            let mut storage: Vec<u8> = Vec::new();
            match files::read_entry_from_file(&mut storage, &mut entry, &dt_label.to_string()) {
                Ok(_) => println!("{}", entry.to_string()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(Box::new(e));
                }
            }
            return Ok(());
        }
        Some(_) | None => {
            let _ = app.print_long_help();
            println!();
            return Ok(());
        }
    };

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
}
