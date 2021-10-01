use arbitrary::{Arbitrary, Unstructured};
use std::error::Error;
use std::fmt;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

// You should only construct a NoNewlines if you know for a fact
// that the contained string has no newlines.
#[derive(Debug, PartialEq)]
pub struct NoNewlines<'a>(&'a str);

pub fn as_no_newlines(s: &str) -> Option<NoNewlines> {
    if s.contains('\n') {
        None
    } else {
        Some(NoNewlines(s))
    }
}

pub fn promise_no_newlines(s: &str) -> NoNewlines {
    match as_no_newlines(s) {
        Some(n) => n,
        None => panic!("promise_no_newlines can't be called with {}", s),
    }
}

impl<'a> fmt::Display for NoNewlines<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let NoNewlines(s) = self;
        write!(f, "{}", s)
    }
}

impl<'a> Arbitrary<'a> for NoNewlines<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<NoNewlines<'a>> {
        let raw = u.arbitrary::<&'a str>()?;
        let clean = match raw.find('\n') {
            Some(ix) => &raw[..ix],
            None => raw,
        };
        Ok(NoNewlines(clean))
    }
}

#[derive(Debug, PartialEq)]
pub struct Observation<'a> {
    pub name: NoNewlines<'a>,
    pub value: NoNewlines<'a>,
}

impl<'a> Arbitrary<'a> for Observation<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Observation<'a>> {
        let NoNewlines(raw_name) = u.arbitrary::<NoNewlines<'a>>()?;
        let value = u.arbitrary::<NoNewlines<'a>>()?;

        let name_text = match raw_name.find(':') {
            Some(ix) => &raw_name[..ix],
            None => raw_name,
        };

        Ok(Observation {
            name: NoNewlines(name_text),
            value,
        })
    }
}

impl<'a> fmt::Display for Observation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", &self.name, &self.value)
    }
}

#[derive(Arbitrary, Debug, PartialEq)]
pub enum Task<'a> {
    Todo(NoNewlines<'a>),
    Working(NoNewlines<'a>),
    Done(NoNewlines<'a>),
    Cancelled(NoNewlines<'a>),
}

impl<'a> fmt::Display for Task<'a> {
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
pub struct Event<'a> {
    pub when: OffsetDateTime,
    pub text: NoNewlines<'a>,
}

const TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!(
    "[year]-[month repr:numerical]-[day] [weekday repr:short] [hour repr:24]:[minute]"
);

impl<'a> fmt::Display for Event<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stamp = self.when.format(&TIMESTAMP_FORMAT).unwrap();
        write!(f, "<{}> {}", stamp, self.text)
    }
}

impl<'a> Arbitrary<'a> for Event<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let text = u.arbitrary::<NoNewlines<'a>>()?;
        let when_stamp = u.int_in_range::<i64>(0..=2147483640)?;
        let when = OffsetDateTime::from_unix_timestamp(when_stamp).unwrap(); // when_stamp is not out of range
        Ok(Event { text, when })
    }
}

#[derive(Debug, PartialEq)]
pub struct Note<'a>(&'a str);

pub fn promise_nonempty_note(s: &str) -> Note {
    if s.is_empty() {
        panic!("promise_nonempty_note called with an empty string");
    }

    Note(s)
}

impl<'a> fmt::Display for Note<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Arbitrary<'a> for Note<'a> {
    // Lots of invariants here, it'd be nice if they were
    // enforced or marked someplace like NoNewlines is?
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let src = u.arbitrary::<&'a str>()?;

        let mut s = match src.find("\n\n") {
            Some(ix) => &src[..ix],
            None => src,
        };

        while !s.is_empty() {
            s = s.trim_matches('\n');

            match consume_task(s) {
                ConsumeResult::Found {
                    remaining,
                    found: _,
                } => {
                    s = remaining;
                    continue;
                }
                ConsumeResult::Problem(_) => {
                    s = &s[1..];
                    continue;
                }
                ConsumeResult::NotFound => (),
            }

            match consume_event(s) {
                ConsumeResult::Found {
                    remaining,
                    found: _,
                } => {
                    s = remaining;
                    continue;
                }
                ConsumeResult::Problem(_) => {
                    s = &s[1..];
                    continue;
                }
                ConsumeResult::NotFound => (),
            }

            break;
        }

        if s.is_empty() {
            s = "x"
        }

        Ok(Note(s))
    }
}

#[derive(Arbitrary, Debug, PartialEq)]
pub struct Entry<'a> {
    pub label: NoNewlines<'a>,
    pub observations: Vec<Observation<'a>>,
    pub tasks: Vec<Task<'a>>,
    pub events: Vec<Event<'a>>,
    pub notes: Vec<Note<'a>>,
}

impl<'a> Default for Entry<'a> {
    fn default() -> Self {
        Entry {
            label: NoNewlines("PLACEHOLDER"),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        }
    }
}

impl<'a> fmt::Display for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "coach1")?;
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
                "coach files must begin with a line containing only the text \"coach1\""
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

pub fn parse<'a>(text: &'a str, dest: &mut Entry<'a>) -> Result<(), ParseError> {
    let mut remaining = text;

    if remaining.starts_with("coach1\n") {
        remaining = &remaining[7..];
    } else {
        return Err(ParseError::NoMagicNumber);
    }

    match remaining.find('\n') {
        Some(0) => return Err(ParseError::EmptyLabel),
        Some(ix) => {
            dest.label = NoNewlines(&remaining[..ix]);
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
                dest.observations.push(found);
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
                dest.tasks.push(found);
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
                dest.events.push(found);
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
                dest.notes.push(found);
            }
            ConsumeResult::Problem(err) => return Err(err),
            ConsumeResult::NotFound => (),
        };
    }

    Ok(())
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
                name: NoNewlines(&obs_line[..ix]),
                value: NoNewlines(&obs_line[ix + 2..]),
            },
        },
        None => ConsumeResult::Problem(ParseError::ExpectedObservation),
    }
}

fn consume_task(remaining: &str) -> ConsumeResult<'_, Task<'_>> {
    let (task_end, rest) = match remaining.find('\n') {
        Some(ix) => (ix, &remaining[ix + 1..]),
        None => (remaining.len(), &remaining[remaining.len()..]),
    };

    if task_end == 0 {
        return ConsumeResult::NotFound;
    }

    let found = match remaining {
        x if x.starts_with("TODO ") => Task::Todo(NoNewlines(&x[5..task_end])),
        x if x.starts_with("WORKING ") => Task::Working(NoNewlines(&x[8..task_end])),
        x if x.starts_with("DONE ") => Task::Done(NoNewlines(&x[5..task_end])),
        x if x.starts_with("CANCELLED ") => Task::Cancelled(NoNewlines(&x[10..task_end])),
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
            text: NoNewlines(body_text.trim_start()),
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
        found: Note(note_text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_empty_entry_to_string() {
        let e = Entry {
            label: NoNewlines("Test"),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        };

        assert_eq!("coach1\nTest\n\n", e.to_string())
    }

    #[test]
    fn test_entry_observations_to_string() {
        let e = Entry {
            label: NoNewlines("Test"),
            observations: vec![
                Observation {
                    name: NoNewlines("key"),
                    value: NoNewlines("value1"),
                },
                Observation {
                    name: NoNewlines("key"),
                    value: NoNewlines("value2"),
                },
            ],
            tasks: vec![],
            events: vec![],
            notes: vec![],
        };

        assert_eq!("coach1\nTest\nkey: value1\nkey: value2\n\n", e.to_string())
    }

    #[test]
    fn test_entry_tasks_to_string() {
        let e = Entry {
            label: NoNewlines("Test"),
            observations: vec![],
            tasks: vec![
                super::Task::Todo(NoNewlines("take a break")),
                Task::Working(NoNewlines("learn rust")),
                Task::Done(NoNewlines("pet the dog")),
                Task::Cancelled(NoNewlines("teach the dog rust")),
            ],
            events: vec![],
            notes: vec![],
        };

        assert_eq!(
            "coach1
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
            label: NoNewlines("Test"),
            observations: vec![],
            tasks: vec![],
            events: vec![
                Event {
                    when: datetime!(2021-10-31 21:00 UTC),
                    text: NoNewlines("working in the lab late one night"),
                },
                Event {
                    when: datetime!(2021-10-31 22:10 UTC),
                    text: NoNewlines("my eyes beheld an eerie sight"),
                },
            ],
            notes: vec![],
        };

        assert_eq!(
            "coach1
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
            label: NoNewlines("Test"),
            observations: vec![],
            tasks: vec![],
            events: vec![],
            notes: vec![
                Note("dogs can't type"),
                Note("It's a good thing\nthe dog learned graffiti\nfrom her palm pilot"),
            ],
        };

        assert_eq!(
            "coach1
Test

dogs can't type

It's a good thing
the dog learned graffiti
from her palm pilot

",
            e.to_string()
        )
    }

    const MESSAGE: &str = "coach1
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
        let mut e = Entry::default();
        let _ = parse(MESSAGE, &mut e).unwrap();
        assert_eq!(NoNewlines("Test"), e.label);
    }

    #[test]
    fn test_parse_tasks() {
        let mut e = Entry::default();
        let _ = parse(MESSAGE, &mut e).unwrap();
        assert_eq!(
            vec![
                Task::Todo(NoNewlines("take a break")),
                Task::Working(NoNewlines("learn rust")),
                Task::Done(NoNewlines("pet the dog")),
                Task::Cancelled(NoNewlines("teach the dog rust")),
            ],
            e.tasks
        );
    }

    #[test]
    fn test_parse_events() {
        let mut e = Entry::default();
        let _ = parse(MESSAGE, &mut e).unwrap();
        assert_eq!(
            vec![
                Event {
                    when: datetime!(2021-10-31 21:10:00 UTC),
                    text: NoNewlines("working in the lab late one night"),
                },
                Event {
                    when: datetime!(2021-10-31 22:10:00 UTC),
                    text: NoNewlines("my eyes beheld an eerie sight"),
                },
            ],
            e.events
        )
    }

    #[test]
    fn test_parse_notes() {
        let mut e = Entry::default();
        let _ = parse(MESSAGE, &mut e).unwrap();
        assert_eq!(
            vec![
                Note("This is note one"),
                Note("And this is note two,\nit is multiline")
            ],
            e.notes
        );
    }

    #[test]
    fn test_parse_just_label() {
        let mut e = Entry::default();
        let _ = parse("coach1\nLabel\n\n", &mut e).unwrap();
        let expect = Entry {
            label: NoNewlines("Label"),
            ..Entry::default()
        };
        assert_eq!(expect, e);
    }

    #[test]
    fn test_display_pure_label() {
        let e = Entry {
            label: NoNewlines("Label"),
            ..Entry::default()
        };
        assert_eq!("coach1\nLabel\n\n", e.to_string());
    }

    #[test]
    fn test_parse_no_terminator() {
        let s = "coach1\nLabel\n\nNo terminator";
        let mut e = Entry::default();
        let _ = parse(s, &mut e);
    }

    #[test]
    fn test_roundtrips() {
        let source = Entry {
            label: NoNewlines("Test"),
            observations: vec![],
            tasks: vec![Task::Working(NoNewlines("Task"))],
            events: vec![],
            notes: vec![],
        };
        let stringed = source.to_string();
        let mut dest = Entry::default();
        let _ = parse(&stringed, &mut dest);
        assert_eq!(source, dest);
    }
}
