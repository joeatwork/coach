#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate coach;

fuzz_target!(|original: coach::entry::Entry<'_>| {
    if original.label.is_empty() {
        return; // Known uninteresting case
    }

    let original_s = original.to_string();
    let mut parsed = coach::entry::Entry::default();
    if let  = coach::entry::parse(&original_s, &mut parsed).unwrap();
    if original.ne(&parsed) {
        panic!("round trip failed:\n{}\n\n{}", original, parsed)
    }
});
