#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to a valid UTF-8 string. If it's not valid UTF-8,
    // skip it—your lexer takes &str, not &[u8].
    if let Ok(s) = std::str::from_utf8(data) {
        // We don't care about the Result. We only care that calling
        // lex() never panics, loops forever, or overflows memory.
        let _ = kowari_lexer::lex(s);
    }
});