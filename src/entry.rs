use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Task<'a> {
    Todo(&'a str),
    Working(&'a str),
    Done(&'a str),
    Cancelled(&'a str),
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

pub struct Entry<'a> {
    pub label: &'a str,
    pub observations: Vec<(&'a str, &'a str)>,
    pub tasks: Vec<Task<'a>>,
    pub notes: Vec<&'a str>,
}

impl Entry<'_> {
    pub fn new() -> Self {
        Entry {
            label: "",
            observations: vec![],
            tasks: vec![],
            notes: vec![],
        }
    }
}

impl<'a> fmt::Display for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.label)?;
        for ob in self.observations.iter() {
            writeln!(f, "{}: {}", ob.0, ob.1)?;
        }
        writeln!(f)?;

        for t in self.tasks.iter() {
            writeln!(f, "{}", t)?;
        }
        if !self.tasks.is_empty() {
            writeln!(f)?;
        }

        for n in self.notes.iter() {
            writeln!(f, "{}", n)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

// TODO: impl Error for ParseError
#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyLabel,
    MissingNewline,
    ExpectedObservation,
}

enum ConsumeResult<'a, T> {
    NotFound,
    Found { remaining: &'a str, found: T },
    Problem(ParseError),
}

pub fn parse<'a>(text: &'a str, dest: &mut Entry<'a>) -> Result<(), ParseError> {
    let mut remaining = text;

    match remaining.find('\n') {
        Some(0) => return Err(ParseError::EmptyLabel),
        Some(ix) => {
            dest.label = &remaining[..ix];
            remaining = &remaining[ix + 1..];
        }
        None => return Err(ParseError::MissingNewline),
    }

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
        }

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
        }
    }

    Ok(())
}

fn consume_observation(remaining: &str) -> ConsumeResult<'_, (&str, &str)> {
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
            found: (&obs_line[..ix], &obs_line[ix + 2..]),
        },
        None => ConsumeResult::Problem(ParseError::ExpectedObservation),
    }
}

// TODO Something kinda gross here - I'd like to return (Task || None || Err)
// which seems like it needs it's own enum...
fn consume_task(remaining: &str) -> ConsumeResult<'_, Task<'_>> {
    let task_end = match remaining.find('\n') {
        Some(ix) => ix,
        None => return ConsumeResult::Problem(ParseError::MissingNewline),
    };

    if task_end == 0 {
        return ConsumeResult::NotFound;
    }

    let found = match remaining {
        x if x.starts_with("TODO ") => Task::Todo(&x[5..task_end]),
        x if x.starts_with("WORKING ") => Task::Working(&x[8..task_end]),
        x if x.starts_with("DONE ") => Task::Done(&x[5..task_end]),
        x if x.starts_with("CANCELLED ") => Task::Cancelled(&x[10..task_end]),
        _ => return ConsumeResult::NotFound,
    };

    ConsumeResult::Found {
        remaining: &remaining[task_end + 1..],
        found,
    }
}

// A note begins with a non-blank line and is terminated either by a blank line or end-of-string.
fn consume_note(remaining: &str) -> ConsumeResult<'_, &str> {
    let mut note_end: usize = 0;
    let mut remain_begin: usize = 0;

    if remaining.is_empty() {
        return ConsumeResult::NotFound;
    }

    if remaining.starts_with('\n') {
        return ConsumeResult::NotFound;
    }

    for (ix, _) in remaining.match_indices('\n') {
        if ix == note_end + 1 {
            remain_begin = ix;
            break;
        }
        note_end = ix;
        remain_begin = note_end;
    }

    if note_end == 0 {
        note_end = remaining.len();
    }

    if note_end == 0 {
        return ConsumeResult::NotFound;
    }

    ConsumeResult::Found {
        remaining: &remaining[remain_begin..],
        found: &remaining[..note_end],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_entry_to_string() {
        let e = Entry {
            label: "Test",
            observations: vec![],
            tasks: vec![],
            notes: vec![],
        };

        assert_eq!("Test\n\n", e.to_string())
    }

    #[test]
    fn test_entry_observations_to_string() {
        let e = Entry {
            label: "Test",
            observations: vec![("key", "value1"), ("key", "value2")],
            tasks: vec![],
            notes: vec![],
        };

        assert_eq!("Test\nkey: value1\nkey: value2\n\n", e.to_string())
    }

    #[test]
    fn test_entry_tasks_to_string() {
        let e = Entry {
            label: "Test",
            observations: vec![],
            tasks: vec![
                super::Task::Todo("take a break"),
                Task::Working("learn rust"),
                Task::Done("pet the dog"),
                Task::Cancelled("teach the dog rust"),
            ],
            notes: vec![],
        };

        assert_eq!(
            "Test

TODO take a break
WORKING learn rust
DONE pet the dog
CANCELLED teach the dog rust

",
            e.to_string()
        )
    }

    #[test]
    fn test_entry_notes_to_string() {
        let e = Entry {
            label: "Test",
            observations: vec![],
            tasks: vec![],
            notes: vec![
                "dogs can't type",
                "It's a good thing\nthe dog learned graffiti\nfrom her palm pilot",
            ],
        };

        assert_eq!(
            "Test

dogs can't type

It's a good thing
the dog learned graffiti
from her palm pilot

",
            e.to_string()
        )
    }

    const MESSAGE: &str = "Test
key: value1
key: value2

TODO take a break
WORKING learn rust
DONE pet the dog
CANCELLED teach the dog rust

This is note one

And this is note two,
it is multiline

";

    #[test]
    fn test_parse_label() {
        let mut e = Entry::new();
        let _ = parse(MESSAGE, &mut e);
        assert_eq!("Test", e.label);
    }

    #[test]
    fn test_parse_tasks() {
        let mut e = Entry::new();
        let _ = parse(MESSAGE, &mut e);
        assert_eq!(
            vec![
                Task::Todo("take a break"),
                Task::Working("learn rust"),
                Task::Done("pet the dog"),
                Task::Cancelled("teach the dog rust"),
            ],
            e.tasks
        );
    }

    #[test]
    fn test_parse_notes() {
        let mut e = Entry::new();
        let _ = parse(MESSAGE, &mut e);
        assert_eq!(
            vec!["This is note one", "And this is note two,\nit is multiline"],
            e.notes
        );
    }

    #[test]
    fn test_parse_missing_parts() {
        // TODO can we just do a table-driven test of parses?
        // Include extra newlines and nonsense too.
    }
}
