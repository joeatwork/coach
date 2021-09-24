#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate coach;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = coach::entry::parse(s);
    }
});
