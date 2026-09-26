#![no_main]

use libfuzzer_sys::fuzz_target;
use omni_lex::Scanner;

fuzz_target!(|data: &[u8]| {
    // No UTF-8 gating: byte entry is the contract, so arbitrary bytes must be
    // scannable without panicking and must still reconstruct exactly.
    let mut scanner = Scanner::new(data, 0);
    let mut reconstructed: Vec<u8> = Vec::with_capacity(data.len() + 16);

    let append = |out: &mut Vec<u8>, span: (usize, usize)| {
        out.extend_from_slice(&data[span.0..span.1]);
    };

    while let Some(token) = scanner.next_token() {
        for trivia in &token.leading_trivia {
            append(&mut reconstructed, (trivia.span.start as usize, trivia.span.end as usize));
        }
        append(&mut reconstructed, (token.span.start as usize, token.span.end as usize));
        for trivia in &token.trailing_trivia {
            append(&mut reconstructed, (trivia.span.start as usize, trivia.span.end as usize));
        }
    }
    for trivia in scanner.take_eof_trivia() {
        append(&mut reconstructed, (trivia.span.start as usize, trivia.span.end as usize));
    }

    if data != reconstructed.as_slice() {
        panic!(
            "lossless reconstruction failed\noriginal:      {:?}\nreconstructed: {:?}",
            data, reconstructed
        );
    }
});
