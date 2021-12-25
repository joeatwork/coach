use arbitrary::{Arbitrary, Unstructured};
use std::error::Error;
use std::fmt;
use std::mem;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

// You should only construct a NoNewlines if you know for a fact
// that the contained string has no newlines.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoNewlines(String);

pub fn as_no_newlines(s: String) -> Option<NoNewlines> {
    if s.contains('\n') {
        None
    } else {
        Some(NoNewlines(s))
    }
}

impl<'a> fmt::Display for NoNewlines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let NoNewlines(s) = self;
        write!(f, "{}", s)
    }
}

fn arbitrary_without_match<'a, F>(
    u: &mut Unstructured<'a>,
    matcher: F,
) -> arbitrary::Result<&'a str>
where
    F: Fn(char) -> bool,
{
    let raw = u.arbitrary::<&'a str>()?;
    let clean = match raw.find(matcher) {
        Some(ix) => &raw[..ix],
        None => raw,
    };
    Ok(clean)
}

impl<'a> Arbitrary<'a> for NoNewlines {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<NoNewlines> {
        let s = arbitrary_without_match(u, |c| c == '\n')?;
        Ok(NoNewlines(s.to_string()))
    }
}

#[derive(Debug, PartialEq)]
pub struct ObservationName(String);

pub fn as_observation_name(s: String) -> Option<ObservationName> {
    if s.is_empty() || s.contains(|c| c == '\n' || c == ':') {
        return None;
    }

    Some(ObservationName(s))
}

impl fmt::Display for ObservationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Arbitrary<'a> for ObservationName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<ObservationName> {
        let s = arbitrary_without_match(u, |c| c == '\n' || c == ':')?;
        Ok(ObservationName(s.to_string()))
    }
}

#[derive(Arbitrary, Debug, PartialEq)]
pub struct Observation {
    pub name: ObservationName,
    pub value: NoNewlines,
}

impl<'a> fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", &self.name, &self.value)
    }
}

#[derive(Arbitrary, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Task {
    Working(NoNewlines),
    Todo(NoNewlines),
    Done(NoNewlines),
    Cancelled(NoNewlines),
}

impl Task {
    pub fn is_live(&self) -> bool {
        match self {
            Task::Todo(_) | Task::Working(_) => true,
            _ => false,
        }
    }

    pub fn message(&mut self) -> &mut NoNewlines {
        match self {
            Task::Todo(s) => s,
            Task::Done(s) => s,
            Task::Working(s) => s,
            Task::Cancelled(s) => s,
        }
    }
}

impl<'a> fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Todo(s) => write!(f, "TODO {}", s),
            Task::Done(s) => write!(f, "DONE {}", s),
            Task::Working(s) => write!(f, "WORKING {}", s),
            Task::Cancelled(s) => write!(f, "CANCELLED {}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub when: OffsetDateTime,
    pub text: NoNewlines,
}

const TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!(
    "[year]-[month repr:numerical]-[day] [weekday repr:short] [hour repr:24]:[minute]"
);

impl<'a> fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stamp = self.when.format(&TIMESTAMP_FORMAT).unwrap();
        write!(f, "<{}> {}", stamp, self.text)
    }
}

impl<'a> Arbitrary<'a> for Event {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let text = u.arbitrary::<NoNewlines>()?;
        let when_stamp = u.int_in_range::<i64>(0..=2147483640)?;
        let when = OffsetDateTime::from_unix_timestamp(when_stamp).unwrap(); // when_stamp is not out of range
        Ok(Event { text, when })
    }
}

#[derive(Debug, PartialEq)]
pub struct Note(String);

pub fn promise_nonempty_note(s: String) -> Note {
    if s.is_empty() {
        panic!("promise_nonempty_note called with an empty string");
    }

    Note(s)
}

pub fn as_note(s: String) -> Option<Note> {
    if s.is_empty() || s.contains("\n\n") || s.starts_with('\n') || s.ends_with('\n') {
        return None;
    }

    match consume_task(&s) {
        ConsumeResult::NotFound => {}
        _ => return None,
    }

    match consume_event(&s) {
        ConsumeResult::NotFound => {}
        _ => return None,
    }

    Some(Note(s))
}

impl<'a> fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Arbitrary<'a> for Note {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let src = u.arbitrary::<&'a str>()?;

        let mut s = match src.find("\n\n") {
            Some(ix) => &src[..ix],
            None => src,
        };

        s = s.trim_end_matches('\n');
        while !s.is_empty() {
            if let Some(n) = as_note(s.to_string()) {
                return Ok(n);
            }
            s = &s[1..];
        }

        Ok(Note(String::from("x")))
    }
}

#[derive(Arbitrary, Debug, PartialEq)]
pub struct Entry {
    pub label: NoNewlines,
    pub observations: Vec<Observation>,
    pub tasks: Vec<Task>,
    pub events: Vec<Event>,
    pub notes: Vec<Note>,
}

impl Entry {
    pub fn update_task<F>(&mut self, ix: usize, updater: F)
    where
        F: FnOnce(NoNewlines) -> Task,
    {
        let old_message = mem::take(self.tasks[ix].message());
        let new_task = updater(old_message);
        let _ = mem::replace(&mut self.tasks[ix], new_task);
    }
}

impl<'a> Default for Entry {
    fn default() -> Self {
        Entry {
            label: NoNewlines(String::from("PLACEHOLDER")),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        }
    }
}

impl<'a> fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#coach")?;
        writeln!(f, "{}", self.label)?;
        for ob in self.observations.iter() {
            writeln!(f, "{}", ob)?;
        }
        writeln!(f)?;

        for t in self.tasks.iter() {
            writeln!(f, "{}", t)?;
        }
        if !self.tasks.is_empty() {
            writeln!(f)?;
        }

        for e in self.events.iter() {
            writeln!(f, "* {}", e)?;
        }
        if !self.events.is_empty() {
            writeln!(f)?;
        }

        for n in self.notes.iter() {
            writeln!(f, "{}", n)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    NoMagicNumber,
    EmptyLabel,
    MissingNewline,
    ExpectedObservation,
    MissingTimestamp,
    MalformedTimestamp,
}

// TODO it'd be nice to have some metadata (input position at least)
// for these errors
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ParseError::NoMagicNumber => {
                "coach files must begin with a line containing only the text \"#coach\""
            }
            ParseError::EmptyLabel => "entries must contain a nonempty first line",
            ParseError::MissingNewline => {
                "newlines are required after the label and observations in an entry"
            }
            ParseError::ExpectedObservation => {
                "there must be a blank line between the entry header and any notes"
            }
            ParseError::MissingTimestamp => "an event was found, but it was missing a <timestamp>",
            ParseError::MalformedTimestamp => {
                "the timestamp for this event was in an unexpected format"
            }
        };
        write!(f, "{}", msg)
    }
}

impl Error for ParseError {}

enum ConsumeResult<'a, T> {
    NotFound,
    Found { remaining: &'a str, found: T },
    Problem(ParseError),
}

pub fn parse(text: &str) -> Result<Entry, ParseError> {
    let mut remaining = text;
    let label: NoNewlines;
    let mut observations: Vec<Observation> = vec![];
    let mut tasks: Vec<Task> = vec![];
    let mut events: Vec<Event> = vec![];
    let mut notes: Vec<Note> = vec![];

    if remaining.starts_with("#coach\n") {
        remaining = &remaining[7..];
    } else {
        return Err(ParseError::NoMagicNumber);
    }

    match remaining.find('\n') {
        Some(0) => return Err(ParseError::EmptyLabel),
        Some(ix) => {
            label = NoNewlines(String::from(&remaining[..ix]));
            remaining = &remaining[ix + 1..];
        }
        None => return Err(ParseError::MissingNewline),
    };

    loop {
        match consume_observation(remaining) {
            ConsumeResult::Found {
                remaining: r,
                found,
            } => {
                observations.push(found);
                remaining = r;
            }
            ConsumeResult::NotFound => break,
            ConsumeResult::Problem(err) => return Err(err),
        }
    }

    while !remaining.is_empty() {
        remaining = remaining.trim_start_matches('\n');

        match consume_task(remaining) {
            ConsumeResult::Found {
                remaining: r,
                found,
            } => {
                remaining = r;
                tasks.push(found);
                continue;
            }
            ConsumeResult::Problem(err) => return Err(err),
            ConsumeResult::NotFound => (),
        };

        match consume_event(remaining) {
            ConsumeResult::Found {
                remaining: r,
                found,
            } => {
                remaining = r;
                events.push(found);
                continue;
            }
            ConsumeResult::Problem(err) => return Err(err),
            ConsumeResult::NotFound => (),
        };

        // If consume_note returns NotFound for anything
        // other than a blank line, the parser will break.
        match consume_note(remaining) {
            ConsumeResult::Found {
                remaining: r,
                found,
            } => {
                remaining = r;
                notes.push(found);
            }
            ConsumeResult::Problem(err) => return Err(err),
            ConsumeResult::NotFound => (),
        };
    }

    Ok(Entry {
        label,
        observations,
        tasks,
        events,
        notes,
    })
}

fn consume_observation(remaining: &str) -> ConsumeResult<'_, Observation> {
    if remaining.is_empty() {
        return ConsumeResult::NotFound;
    }

    if remaining.starts_with('\n') {
        return ConsumeResult::NotFound;
    }

    let obs_end = match remaining.find('\n') {
        Some(ix) => ix,
        None => return ConsumeResult::Problem(ParseError::MissingNewline),
    };

    let obs_line = &remaining[0..obs_end];
    match obs_line.find(": ") {
        Some(ix) => ConsumeResult::Found {
            remaining: &remaining[obs_end + 1..],
            found: Observation {
                name: ObservationName(String::from(&obs_line[..ix])),
                value: NoNewlines(String::from(&obs_line[ix + 2..])),
            },
        },
        None => ConsumeResult::Problem(ParseError::ExpectedObservation),
    }
}

fn consume_task(remaining: &str) -> ConsumeResult<'_, Task> {
    let (task_end, rest) = match remaining.find('\n') {
        Some(ix) => (ix, &remaining[ix + 1..]),
        None => (remaining.len(), &remaining[remaining.len()..]),
    };

    if task_end == 0 {
        return ConsumeResult::NotFound;
    }

    let found = match remaining {
        x if x.starts_with("TODO ") => Task::Todo(NoNewlines(String::from(&x[5..task_end]))),
        x if x.starts_with("WORKING ") => Task::Working(NoNewlines(String::from(&x[8..task_end]))),
        x if x.starts_with("DONE ") => Task::Done(NoNewlines(String::from(&x[5..task_end]))),
        x if x.starts_with("CANCELLED ") => {
            Task::Cancelled(NoNewlines(String::from(&x[10..task_end])))
        }
        _ => return ConsumeResult::NotFound,
    };

    ConsumeResult::Found {
        remaining: rest,
        found,
    }
}

fn consume_event(remaining: &str) -> ConsumeResult<'_, Event> {
    let (line_end, rest) = match remaining.find('\n') {
        Some(ix) => (ix, &remaining[ix + 1..]),
        None => (remaining.len(), &remaining[remaining.len()..]),
    };

    if line_end == 0 {
        return ConsumeResult::NotFound;
    }

    if !remaining.starts_with("* ") {
        return ConsumeResult::NotFound;
    }

    let eventline = &remaining[2..line_end];

    if !eventline.starts_with('<') {
        // TODO in the future, maybe make timestamps optional?
        // TODO if they aren't optional, why do we need the leading asterisk?
        return ConsumeResult::Problem(ParseError::MissingTimestamp);
    }

    let (when_text, body_text) = match eventline.find('>') {
        Some(ix) => (&eventline[1..ix], &eventline[ix + 1..]),
        None => {
            return ConsumeResult::Problem(ParseError::MalformedTimestamp);
        }
    };

    let dt = match PrimitiveDateTime::parse(when_text.trim(), &TIMESTAMP_FORMAT) {
        Ok(d) => d.assume_offset(UtcOffset::UTC),
        Err(_) => {
            return ConsumeResult::Problem(ParseError::MalformedTimestamp);
        }
    };

    ConsumeResult::Found {
        found: Event {
            text: NoNewlines(String::from(body_text.trim_start())),
            when: dt,
        },
        remaining: rest,
    }
}

// A note begins with a non-blank line and is terminated either by a blank line or end-of-string.
fn consume_note(remaining: &str) -> ConsumeResult<'_, Note> {
    if remaining.is_empty() {
        return ConsumeResult::NotFound;
    }

    if remaining.starts_with('\n') {
        return ConsumeResult::NotFound;
    }

    let (note_text, ret_remain) = match remaining.find("\n\n") {
        Some(ix) => (&remaining[..ix], &remaining[ix + 1..]),
        None => (remaining, &remaining[remaining.len()..]),
    };

    if note_text.is_empty() {
        return ConsumeResult::NotFound;
    }

    ConsumeResult::Found {
        remaining: ret_remain,
        found: Note(String::from(note_text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_empty_entry_to_string() {
        let e = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        };

        assert_eq!("#coach\nTest\n\n", e.to_string())
    }

    #[test]
    fn test_entry_observations_to_string() {
        let e = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![
                Observation {
                    name: ObservationName(String::from("key")),
                    value: NoNewlines(String::from("value1")),
                },
                Observation {
                    name: ObservationName(String::from("key")),
                    value: NoNewlines(String::from("value2")),
                },
            ],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        };

        assert_eq!("#coach\nTest\nkey: value1\nkey: value2\n\n", e.to_string())
    }

    #[test]
    fn test_entry_tasks_to_string() {
        let e = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![],
            tasks: vec![
                super::Task::Todo(NoNewlines(String::from("take a break"))),
                Task::Working(NoNewlines(String::from("learn rust"))),
                Task::Done(NoNewlines(String::from("pet the dog"))),
                Task::Cancelled(NoNewlines(String::from("teach the dog rust"))),
            ],
            events: vec![],
            notes: vec![],
        };

        assert_eq!(
            "#coach
Test

TODO take a break
WORKING learn rust
DONE pet the dog
CANCELLED teach the dog rust

",
            e.to_string()
        )
    }

    #[test]
    fn test_entry_events_to_string() {
        let e = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![],
            tasks: vec![],
            events: vec![
                Event {
                    when: datetime!(2021-10-31 21:00 UTC),
                    text: NoNewlines(String::from("working in the lab late one night")),
                },
                Event {
                    when: datetime!(2021-10-31 22:10 UTC),
                    text: NoNewlines(String::from("my eyes beheld an eerie sight")),
                },
            ],
            notes: vec![],
        };

        assert_eq!(
            "#coach
Test

* <2021-10-31 Sun 21:00> working in the lab late one night
* <2021-10-31 Sun 22:10> my eyes beheld an eerie sight

",
            e.to_string()
        )
    }

    #[test]
    fn test_entry_notes_to_string() {
        let e = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![
                Note(String::from("dogs can't type")),
                Note(String::from(
                    "It's a good thing\nthe dog learned graffiti\nfrom her palm pilot",
                )),
            ],
        };

        assert_eq!(
            "#coach
Test

dogs can't type

It's a good thing
the dog learned graffiti
from her palm pilot

",
            e.to_string()
        )
    }

    const MESSAGE: &str = "#coach
Test
key: value1
key: value2

TODO take a break
WORKING learn rust
DONE pet the dog
CANCELLED teach the dog rust

* <2021-10-31 Sun 21:10> working in the lab late one night
* <2021-10-31 Sun 22:10> my eyes beheld an eerie sight

This is note one

And this is note two,
it is multiline

";

    #[test]
    fn test_parse_label() {
        let e = parse(MESSAGE).unwrap();
        assert_eq!(NoNewlines(String::from("Test")), e.label);
    }

    #[test]
    fn test_parse_tasks() {
        let e = parse(MESSAGE).unwrap();
        assert_eq!(
            vec![
                Task::Todo(NoNewlines(String::from("take a break"))),
                Task::Working(NoNewlines(String::from("learn rust"))),
                Task::Done(NoNewlines(String::from("pet the dog"))),
                Task::Cancelled(NoNewlines(String::from("teach the dog rust"))),
            ],
            e.tasks
        );
    }

    #[test]
    fn test_parse_events() {
        let e = parse(MESSAGE).unwrap();
        assert_eq!(
            vec![
                Event {
                    when: datetime!(2021-10-31 21:10:00 UTC),
                    text: NoNewlines(String::from("working in the lab late one night")),
                },
                Event {
                    when: datetime!(2021-10-31 22:10:00 UTC),
                    text: NoNewlines(String::from("my eyes beheld an eerie sight")),
                },
            ],
            e.events
        )
    }

    #[test]
    fn test_parse_notes() {
        let e = parse(MESSAGE).unwrap();
        assert_eq!(
            vec![
                Note(String::from("This is note one")),
                Note(String::from("And this is note two,\nit is multiline")),
            ],
            e.notes
        );
    }

    #[test]
    fn test_parse_just_label() {
        let e = parse("#coach\nLabel\n\n").unwrap();
        let expect = Entry {
            label: NoNewlines(String::from("Label")),
            ..Entry::default()
        };
        assert_eq!(expect, e);
    }

    #[test]
    fn test_display_pure_label() {
        let e = Entry {
            label: NoNewlines(String::from("Label")),
            ..Entry::default()
        };
        assert_eq!("#coach\nLabel\n\n", e.to_string());
    }

    #[test]
    fn test_parse_no_terminator() {
        let s = "#coach\nLabel\n\nNo terminator";
        let _ = parse(s).unwrap();
    }

    #[test]
    fn test_roundtrips() {
        let source = Entry {
            label: NoNewlines(String::from("Test")),
            observations: vec![],
            tasks: vec![Task::Working(NoNewlines(String::from("Task")))],
            events: vec![],
            notes: vec![],
        };
        let stringed = source.to_string();
        let dest = parse(&stringed).unwrap();
        assert_eq!(source, dest);
    }
}
