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
const MAX_ENTRY_SIZE_BYTES: usize = 32 * 1024;

const DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month repr:numerical]-[day]");

fn no_newline_validator(val: String) -> Result<(), String> {
    match entry::as_no_newlines(val) {
        Some(_) => Ok(()),
        None => Err(String::from("argument can't contain newlines")),
    }
}

fn observation_name_validator(val: String) -> Result<(), String> {
    match entry::as_observation_name(val) {
        Some(_) => Ok(()),
        None => Err(String::from("observation names must contain at least one character, and can't contain newlines or colons")),
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
    let dt_label = entry::as_no_newlines(dt_formatted).unwrap();

    match matches.subcommand() {
        ("today", Some(_)) => {
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
        ("cat", Some(_)) => {
            let entry = entry_from_file(&dt_label.to_string())?;
            print!("{}", entry);
        }
        ("observe", Some(args)) => {
            let name_str = args.value_of("NAME").unwrap();
            let value_str = args.value_of("VALUE").unwrap();
            let filename = dt_label.to_string();

            let name = entry::as_observation_name(String::from(name_str)).unwrap();
            let value = entry::as_no_newlines(String::from(value_str)).unwrap();

            let mut entry = entry_from_file(&filename)?;
            entry.observations.push(entry::Observation { name, value });

            // TODO: it'd be safer to write to a temp file and then
            // copy it over rather than truncate and write here.
            let mut newfile = OpenOptions::new()
                .write(true)
                .create_new(false)
                .open(&filename)?;
            newfile.write_all(entry.to_string().as_bytes())?;
            newfile.sync_all()?;
        }
        _ => {
            let _ = app.print_long_help();
            println!();
        }
    };

    Ok(())
}

fn entry_from_file(filename: &str) -> Result<entry::Entry, Box<dyn Error>> {
    let mut buf: Vec<u8> = Vec::new();
    let text = files::read_bounded_str_from_file(&mut buf, filename, MAX_ENTRY_SIZE_BYTES)?;
    match entry::parse(text) {
        Ok(e) => Ok(e),
        Err(e) => Err(Box::new(e)),
    }
}
