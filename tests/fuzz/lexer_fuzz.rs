#![no_main]

use libfuzzer_sys::fuzz_target;
use omni_lex::Scanner;
use omni_source::Cursor;

fuzz_target!(|data: &[u8]| {
    // Omni requires valid UTF-8 source files
    let Ok(source_str) = std::str::from_utf8(data) else {
        return; 
    };

    let cursor = Cursor::new(data);
    let mut scanner = Scanner::new(source_str, cursor, 0);
    let mut reconstructed = String::new();

    while let Some(token) = scanner.next_token() {
        // 1. Append leading trivia
        for trivia in &token.leading_trivia {
            reconstructed.push_str(&source_str[trivia.span.start as usize .. trivia.span.end as usize]);
        }
        
        // 2. Append token text
        reconstructed.push_str(&source_str[token.span.start as usize .. token.span.end as usize]);
        
        // 3. Append trailing trivia
        for trivia in &token.trailing_trivia {
            reconstructed.push_str(&source_str[trivia.span.start as usize .. trivia.span.end as usize]);
        }
    }

    // Mathematical proof of LEX-0002
    if source_str != reconstructed {
        panic!(
            "LEX-0002 Lossless Reconstruction Failed!\nOriginal: {:?}\nReconstructed: {:?}", 
            source_str, reconstructed
        );
    }
});
