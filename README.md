# coach

`coach` is a simple, human readable file format for keeping a productivity journal, and a command line tool for managing those files.

## Quick start

```console
# every morning before work, create a new journal entry with 'coach today'
$ coach today
2021-10-06

# As you plan for new tasks throughout your day, you can add them to your TODO list
$ coach task new "write a README"

# You can see all of your tasks for the day
$ coach task
1: TODO implement notes with editor popup
2: TODO clean up README
3: TODO write good --help messages for commands

# you can mark a task as done using its index
$ coach task done 2

# now you can see your changes in your to-do list
$ coach task
1: TODO implement notes with editor popup
2: TODO write good --help messages for commands
3: DONE clean up README

```

## The coach file format

Coach files are short, human readable daily journal files. You can create a new file for
the day by running

```console
$ coach today
```

Coach files contain the following information:

- A _label_ that you can use as a name for this journal entry. The command line tool uses the current date as the label, and as the file name, for the files it creates.
- A list of _observations_. These are key value pairs you can use to track key metrics from day to day, like remaining budget, time until deadlines, or whatever else might be interesting to keep track of over time.
- A list of _tasks_, that you can use to keep track of things you've done, and things that need doing.
- A list of _events_, which are timestamped messages you can use for simple time tracking or to record a log of calls or incidents through your day.
- A collection of _notes_, which are free-form text paragraphs you can use as a journal.

A typical coach file might look like this:

```txt
#coach
2021-10-31
weather: sunny, but windy!
days until Halloween: 0

WORKING write README for coach
TODO put out Halloween lawn decorations
DONE buy candy for trick'r'treaters
CANCELLED dress the dog in her Halloween costume

* <2021-10-31 Sun 10:03> Bought candy at the drug store. Hope the kids like Oops-all-banana Runts!
* <2021-10-31 Sun 13:32> Dog refuses to wear the costume, says it's "demeaning"

This is a sample coach file! All coach files begin with a line just containing
the words "#coach", followed by a label for the file. By default, the label
is the name of the file, which by default is the current date. After the
"#coach" line and the label are observations - a list of key / value pairs,
with the keys and values separated by a colon and space, each on their own line.

After the observations, there is a blank line, and then a list of tasks, each on
their own line. Each task begins with one of the words TODO, WORKING, DONE, or CANCELLED.

After the tasks come events, also one line per event. Events begin with an asterisk
and a timestamp, like "* <1972-06-13 Fri 14:03>".

Finally, there is a list of notes (like this one.) Notes are separated by blank lines.
Notes can't begin with TODO, WORKING, DONE, CANCELLED, or the three characters "* <"

```

The only required parts of the coach file are the first line and the label.

## The coach command line tool

For up to date information about coach commands and options, you can run

```console
$ coach help
```

The command line tool has sub commands that allow you to create and manage
observations, tasks, events, and notes.

To create a new entry, run

```console
$ coach today
```

`coach today` will create a file named after the current date in your
current working directory. The rest of the `coach` commands assume that
a file using this naming format (and named for the current system date)
exists.

If you have a file from the previous day with unfinished tasks you'd like
to move into your new entry, you can use

```console
$ coach today --from_yesterday
```

To add observations, use `coach observe <key> <value>`. For example, you
could record the current weather with something like

```console
$ coach observe weather "sunny, but windy!"
```

To add tasks, you can use `coach task new <task text>`. For example

```sh
$ coach task new "put out Halloween lawn decorations"
```

To review the tasks in the current entry, use `coach task`, which will
print out a list of tasks along with an index number.

```console
$ coach task
1: TODO write README
2: TODO put out Halloween lawn decorations
3: TODO buy candy for trick r treaters
```

You can then use the index numbers to change the state of your tasks. For example,
once the decorations have been put out, you might type

```console
$ coach task done 2
```

Having changed the state of task 2 from TODO to DONE, you will see the following if you run `coach task` again:

```console
$ coach task
1: TODO write README
2: DONE put out Halloween lawn decorations
3: TODO
```

You can record events in your journal entry with `coach event <MESSAGE>`. For example, you might write

```console
$ coach event 'bought candy at the drug store. Hope the kids like Oops-all-banana runts!'
```

To record a note in your journal entry, run

```console
$ coach note
```

which will open a text editor (vi by default, or $EDITOR), and let you write notes.
Paragraphs separated by blank lines in the notes editor will show up as different
notes in your entry.

You can see your whole daily entry with

```console
$ coach cat
```

which will print the current entry to standard out.

## Build and test

You can build coach with

```sh
$ cargo build
```

And run unit tests with

```sh
$ cargo test
```

coach also ships with some simple fuzz tests, but at this writing they are broken in rust nightly.

To try and run the fuzz tests, use

```sh
$ cargo +nightly fuzz run parser
```

or

```sh
$ cargo +nightly fuzz run entry_roundtrip
```
