#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate coach;

fuzz_target!(|original: coach::entry::Entry<'_>| {
    let original_s = original.to_string();
    let mut parsed = coach::entry::Entry::new();
    let _ = coach::entry::parse(&original_s, &mut parsed).unwrap();
    let parsed_s = parsed.to_string();

    if original_s != parsed_s {
        panic!("round trip failed:\n{}\n\n{}", original_s, parsed_s)
    }
});
