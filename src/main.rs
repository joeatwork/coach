use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{Duration, OffsetDateTime};

use coach::editor;
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
        .long_about(
            "Coach is a semi-structured productivity journal file format, and a command
line tool for managing coach files. You can use coach to keep track of a daily
TODO list, keep a record of observations of key metrics, and keep daily 
progress notes.",
        )
        .arg(
            Arg::with_name("entry")
                .long("entry")
                .short("f")
                .takes_value(true)
                .value_name("FILENAME")
                .help("filename of entry to use. If not provided, use a file named after the current UTC date in the current working directory"),
        )
        .arg(
            Arg::with_name("yesterday").long("yesterday").takes_value(false).conflicts_with("entry").help("use the entry named by the previous day, in UTC"),
        )
        .subcommand(
            SubCommand::with_name("today")
                .about("creates a new journal file in the current working directory")
                .long_about(
                    "today will create a new daily entry file in the current working directory,
named after the current date. Other commands will write to or edit that file.",
                ).arg(
                Arg::with_name("from_file")
                .long("from_file")
                .short("r")
                .takes_value(true)
                .value_name("FILENAME")
                .help("migrate TODO and WORKING tasks from FILENAME")
            ).arg(
                Arg::with_name("from_yesterday")
                .long("from_yesterday")
                .short("m")
                .takes_value(false)
                .conflicts_with("from_file")
                .help("migrate TODO and WORKING tasks from a file named after yesterday UTC in the current directory")
            )
        )
        .subcommand(
            SubCommand::with_name("cat")
                .about("writes the contents of the current journal entry to standard out"),
        )
        .subcommand(
            SubCommand::with_name("observe")
                .about("adds a key/value observation to the journal")
                .long_about(
                    "coach observe adds a key / value pair to the current journal entry. You can
use observations to keep track of key project metrics over time. For example,
to add an observation about the weather to your entry, you could use:

    coach observe weather \"bright and sunny\"

To see a list of all of the observations in the current entry, use

    coach observe
",
                )
                .arg(
                    Arg::with_name("NAME")
                        .requires("VALUE")
                        .validator(observation_name_validator)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .validator(no_newline_validator)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("task")
                .about("manages the TODO list from this entry")
                .long_about(
                    "You can use coach task to create new tasks on the to-do list for the current
entry, view the day's tasks, and change the state of existing tasks.

coach tasks are either TODO (you need to get to them), DONE (already completed),
WORKING (this task is in progress), or CANCELLED (you've changed your mind
about doing the task.). You can list all of an entry's tasks with:

    coach task

You can make changes to individual tasks using their indexes - for example,
you can set the second task listed by 'coach task' to DONE with:

    coach task done 2
",
                )
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
                        .about("mark a task as TODO")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("working")
                        .about("mark a task as WORKING")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("done")
                        .about("mark a task as DONE")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("cancel")
                        .about("mark a task as CANCELLED")
                        .arg(Arg::with_name("INDEX").required(true).index(1)),
                ),
        )
        .subcommand(
            SubCommand::with_name("event")
                .about("lists events, or makes note of a new event")
                .long_about(
                    "Coach events are brief notes that include a timestamp. You can use them for
simple time tracking, or to check in during your work. To list all of the
events in an entry, use:

    coach event

To make note of a new event, include a message as an argument, like this:

    coach event \"wrote about text for the event command\"
",
                )
                .arg(
                    Arg::with_name("MESSAGE")
                        .validator(no_newline_validator)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("note")
                .about("add a note to this entry")
                .long_about(
                    "note will open a text editor and allow you to add one or more notes,
to the current entry. You can separate notes by blank lines.",
                ).arg(
                    Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .takes_value(true)
                    .value_name("MESSAGE")
                    .help("if provided, use the argument value for the note content rather than opening an editor")
                ),
        )
        .subcommand(
            SubCommand::with_name("edit").about("opens the current coach entry with a text editor. This could corrupt your file, so be careful!"),
        );
    let matches = app.clone().get_matches();

    let when: OffsetDateTime =
        OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let dt_formatted = when.format(&DATE_FORMAT)?;
    let dt_label = entry::as_no_newlines(dt_formatted).unwrap();
    let day = Duration::new(/* seconds = */ 60 * 60 * 24, 0);
    let yesterday = when.checked_sub(day).unwrap();
    let yesterday_formatted = yesterday.format(&DATE_FORMAT).unwrap();
    let yesterday_label = entry::as_no_newlines(yesterday_formatted).unwrap();

    let entryname = matches
        .value_of("entry")
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            if matches.is_present("yesterday") {
                yesterday_label.to_string()
            } else {
                dt_label.to_string()
            }
        });

    match matches.subcommand() {
        ("today", Some(args)) => {
            let source = args.value_of("from_file").map(|v| v.to_string()).or_else(|| {
                if args.is_present("from_yesterday") {
                    Some(yesterday_label.to_string())
                } else {
                    None
                }
            });

            migrate(source, &entryname)?;
        }
        ("cat", Some(_)) => {
            let entry = files::entry_from_file(&entryname, MAX_ENTRY_SIZE_BYTES)?;
            print!("{}", entry);
        }
        ("observe", Some(args)) => match args.value_of("NAME") {
            Some(name_str) => {
                let value_str = args.value_of("VALUE").unwrap();
                let name = entry::as_observation_name(name_str.to_string()).unwrap();
                let value = entry::as_no_newlines(value_str.to_string()).unwrap();
                observe(&entryname, name, value)?;
            }
            None => {
                let entry = files::entry_from_file(&entryname, MAX_ENTRY_SIZE_BYTES)?;
                for ob in entry.observations {
                    println!("{}", ob);
                }
            }
        },
        ("task", Some(args)) => match args.subcommand() {
            ("new", Some(args)) => {
                let message = args.value_of("MESSAGE").unwrap();
                let message = entry::as_no_newlines(message.to_string()).unwrap();
                new_task(&entryname, message)?;
            }
            ("todo", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&entryname, ix_arg, entry::Task::Todo)?;
            }
            ("done", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&entryname, ix_arg, entry::Task::Done)?;
            }
            ("cancel", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&entryname, ix_arg, entry::Task::Cancelled)?;
            }
            ("working", Some(args)) => {
                let ix_arg = args.value_of("INDEX").unwrap();
                let ix_arg: usize = ix_arg.parse()?;
                update_task(&entryname, ix_arg, entry::Task::Working)?;
            }
            _ => {
                let entry = files::entry_from_file(&entryname, MAX_ENTRY_SIZE_BYTES)?;
                for (ix, t) in entry.tasks.iter().enumerate() {
                    println!("{}: {}", ix + 1, t)
                }
            }
        },
        ("event", Some(args)) => {
            let mut entry = files::entry_from_file(&entryname, MAX_ENTRY_SIZE_BYTES)?;
            match args.value_of("MESSAGE") {
                Some(msg) => {
                    let text = entry::as_no_newlines(msg.to_string()).unwrap();
                    let event = entry::Event::Moment { when, text };
                    println!("{}", event);
                    entry.events.push(event);
                    files::entry_to_file(&entryname, &entry)?;
                }
                None => {
                    for e in entry.events {
                        println!("{}", e);
                    }
                }
            }
        }
        ("note", Some(args)) => {
            let mut entry = files::entry_from_file(&entryname, MAX_ENTRY_SIZE_BYTES)?;
            let text = match args.value_of("message") {
                Some(msg) => String::from(msg),
                None => editor::edit_prompt()?,
            };
            let text = text.trim_matches('\n');
            for body in text.split("\n\n") {
                match entry::as_note(String::from(body)) {
                    Some(n) => entry.notes.push(n),
                    None => {
                        return Err(Box::new(CommandError {
                            desc: String::from(
                                "notes must be nonempty and must not look like events or tasks",
                            ),
                        }))
                    }
                }
            }
            files::entry_to_file(&entryname, &entry)?;
        }
        ("edit", _) => {
            editor::launch_editor(&entryname)?;
        }
        _ => {
            let _ = app.print_long_help();
            println!();
        }
    };

    Ok(())
}

fn migrate(source: Option<String>, toname: &str) -> Result<(), Box<dyn Error>> {
    let mut new = entry::Entry {
        label: entry::as_no_newlines(String::from(toname)).unwrap(),
        ..entry::Entry::default()
    };

    if let Some(fromname) = source {
        let mut old = files::entry_from_file(&fromname, MAX_ENTRY_SIZE_BYTES)?;
        let (live, dead): (Vec<entry::Task>, Vec<entry::Task>) =
            old.tasks.drain(..).partition(|t| t.is_incomplete());

        old.tasks.extend(dead);
        new.tasks.extend(live);

        files::new_entry_file(&toname, &new)?;
        files::entry_to_file(&fromname, &old)?;

        println!("from {} ({} migrated)", fromname, new.tasks.len());
        for task in new.tasks {
            println!("{}", &task);
        }
    } else {
        files::new_entry_file(&toname, &new)?;
    }

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
    entry.tasks.sort();

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

    entry.tasks.sort();
    files::entry_to_file(filename, &entry)?;

    Ok(())
}
