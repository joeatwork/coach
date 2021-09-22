use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

mod entry;

const DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month repr:numerical]-[day]");

fn main() -> Result<(), Box<dyn Error>> {
    // TODO get day from args if not provided
    // TODO testing daytimes?
    // TODO handling a pile of files?
    let moment = SystemTime::now();
    let dt: OffsetDateTime = moment.into();
    let dt_label = dt.format(&DATE_FORMAT)?;

    let mut today = File::create(&dt_label)?;

    let sample = entry::Entry {
        label: &dt_label,
        observations: vec![("example", "this is an example entry")],
        events: vec![entry::Event {
            when: dt,
            text: "created a cool new file",
        }],
        tasks: vec![
            entry::Task::Done("Write an example entry"),
            entry::Task::Todo("Read an entry from a file"),
        ],
        notes: vec!["Notes go here, after observations and tasks"],
    };

    today.write_all(b"coach1\n")?;
    today.write_all(sample.to_string().as_bytes())?;
    today.sync_all()?;
    Ok(())
}
