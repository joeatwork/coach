use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::fmt;
use std::fmt::Display;
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

#[derive(Debug)]
struct CommandError {
    desc: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", &self.desc)
    }
}

impl Error for CommandError {
    fn description(&self) -> &str {
        &self.desc
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        None
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
        )
        .subcommand(
            SubCommand::with_name("task")
                .about("manages the TODO list from this entry")
                .subcommand(
                    SubCommand::with_name("new").about("create a new task").arg(
                        Arg::with_name("MESSAGE")
                            .required(true)
                            .validator(no_newline_validator)
                            .index(1),
                    ),
                )
                .subcommand(
                    SubCommand::with_name("todo")
                        .about("mark an item on the todo list as TODO")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("working")
                        .about("mark an item on the todo list as WORKING")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("done")
                        .about("mark an item on the todo list as DONE")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("cancel")
                        .about("mark an item on the todo list as CANCELLED")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                ),
        )
        .subcommand(
            SubCommand::with_name("event")
                .about("lists events, or makes note of a new event")
                .arg(
                    Arg::with_name("MESSAGE")
                        .validator(no_newline_validator)
                        .index(1),
                ),
        );
    let matches = app.clone().get_matches();

    let moment = SystemTime::now();
    let when: OffsetDateTime = moment.into();
    let dt_formatted = when.format(&DATE_FORMAT)?;
    let dt_label = entry::as_no_newlines(dt_formatted).unwrap();

    let filename = dt_label.to_string();

    match matches.subcommand() {
        ("today", Some(_)) => {
            let entry = entry::Entry {
                label: dt_label,
                ..entry::Entry::default()
            };
            files::new_entry_file(&filename, &entry)?;
        }
        ("cat", Some(_)) => {
            let entry = files::entry_from_file(&dt_label.to_string(), MAX_ENTRY_SIZE_BYTES)?;
            print!("{}", entry);
        }
        ("observe", Some(args)) => {
            let name_str = args.value_of("NAME").unwrap();
            let value_str = args.value_of("VALUE").unwrap();
            let name = entry::as_observation_name(name_str.to_string()).unwrap();
            let value = entry::as_no_newlines(value_str.to_string()).unwrap();
            observe(&filename, name, value)?
        }
        ("task", Some(args)) => match args.subcommand() {
            ("new", Some(args)) => {
                let message = args.value_of("MESSAGE").unwrap();
                let message = entry::as_no_newlines(message.to_string()).unwrap();
                new_task(&filename, message)?;
            }
            ("todo", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&filename, ix_arg, entry::Task::Todo)?;
            }
            ("done", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&filename, ix_arg, entry::Task::Done)?;
            }
            ("cancel", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&filename, ix_arg, entry::Task::Cancelled)?;
            }
            ("working", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&filename, ix_arg, entry::Task::Working)?;
            }
            _ => {
                let entry = files::entry_from_file(&filename, MAX_ENTRY_SIZE_BYTES)?;
                for (ix, t) in entry.tasks.iter().enumerate() {
                    println!("{}: {}", ix + 1, t)
                }
            }
        },
        ("event", Some(args)) => {
            let mut entry = files::entry_from_file(&filename, MAX_ENTRY_SIZE_BYTES)?;
            match args.value_of("MESSAGE") {
                Some(msg) => {
                    let text = entry::as_no_newlines(msg.to_string()).unwrap();
                    let event = entry::Event { when, text };
                    println!("{}", event);
                    entry.events.push(event);
                    files::entry_to_file(&filename, &entry)?;
                }
                None => {
                    for e in entry.events {
                        println!("{}", e);
                    }
                }
            }
        }
        _ => {
            let _ = app.print_long_help();
            println!();
        }
    };

    Ok(())
}

fn observe(
    filename: &str,
    name: entry::ObservationName,
    value: entry::NoNewlines,
) -> Result<(), Box<dyn Error>> {
    let mut entry = files::entry_from_file(filename, MAX_ENTRY_SIZE_BYTES)?;
    let observation = entry::Observation { name, value };
    println!("{}", observation);
    entry.observations.push(observation);

    files::entry_to_file(filename, &entry)?;

    Ok(())
}

fn new_task(filename: &str, message: entry::NoNewlines) -> Result<(), Box<dyn Error>> {
    let mut entry = files::entry_from_file(filename, MAX_ENTRY_SIZE_BYTES)?;
    let task = entry::Task::Todo(message);
    println!("{}", &task);
    entry.tasks.push(task);

    files::entry_to_file(filename, &entry)?;

    Ok(())
}

fn update_task<F>(filename: &str, ix_plus_one: usize, updater: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(entry::NoNewlines) -> entry::Task,
{
    if ix_plus_one == 0 {
        return Err(Box::new(CommandError {
            desc: String::from("task indexes start at 1"),
        }));
    }

    let ix = ix_plus_one - 1;

    let mut entry = files::entry_from_file(filename, MAX_ENTRY_SIZE_BYTES)?;
    if ix >= entry.tasks.len() {
        return Err(Box::new(CommandError {
            desc: format!("{} is to large, no task found", ix_plus_one),
        }));
    }

    entry.update_task(ix, updater);

    println!("{}", entry.tasks[ix]);
    files::entry_to_file(filename, &entry)?;

    Ok(())
}
