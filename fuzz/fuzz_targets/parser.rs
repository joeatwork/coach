#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate coach;

fuzz_target!(|s: &str| {
    let _ = coach::entry::parse(s);
});
