#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate coach;

fuzz_target!(|original: coach::entry::Entry| {
    if original.label.to_string().is_empty() {
        return; // non-interesting case.
    }

    let original_s = original.to_string();
    let parsed = coach::entry::parse(&original_s).unwrap();

    if original.label != parsed.label {
        panic!(
            "round trip failed for label:\n<{}>\n|{}|\n<{:?}>\n|{:?}|\n",
            original, parsed, original, parsed
        )
    }

    if original.observations != parsed.observations {
        panic!(
            "round trip failed for observations:\n<{}>\n|{}|\n<{:?}>\n|{:?}|\n",
            original, parsed, original, parsed
        )
    }

    if original.tasks != parsed.tasks {
        panic!(
            "round trip failed for tasks:\n<{}>\n|{}|\n<{:?}>\n|{:?}|\n",
            original, parsed, original, parsed
        )
    }

    /* TODO I suspect I don't know what "!=" means here for dates, so
    we're not comparing events yet. */

    if original.notes != parsed.notes {
        panic!(
            "round trip failed for notes:\n<{}>\n|{}|\n<{:?}>\n|{:?}|\n",
            original, parsed, original, parsed
        )
    }
});
