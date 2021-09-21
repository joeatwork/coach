use std::fmt;

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
    pub observations: &'a [(&'a str, &'a str)],
    pub tasks: &'a [Task<'a>],
    pub notes: &'a [&'a str],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_entry_to_string() {
        let e = Entry {
            label: "Test",
            observations: &[],
            tasks: &[],
            notes: &[],
        };

        assert_eq!("Test\n\n", e.to_string())
    }

    #[test]
    fn test_entry_observations_to_string() {
        let e = Entry {
            label: "Test",
            observations: &[("key", "value1"), ("key", "value2")],
            tasks: &[],
            notes: &[],
        };

        assert_eq!("Test\nkey: value1\nkey: value2\n\n", e.to_string())
    }

    #[test]
    fn test_entry_tasks_to_string() {
        let e = Entry {
            label: "Test",
            observations: &[],
            tasks: &[
                super::Task::Todo("take a break"),
                Task::Working("learn rust"),
                Task::Done("pet the dog"),
                Task::Cancelled("teach the dog rust"),
            ],
            notes: &[],
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
            observations: &[],
            tasks: &[],
            notes: &[
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
}
