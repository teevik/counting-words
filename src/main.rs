use tracing::info;

use crate::generate::generate_corpus;

mod generate;

fn is_whitespace(byte: u8) -> bool {
    match byte {
        b' ' | b'\n' | b'\r' | b'\t' | b'\x0b' | b'\x0c' => true,
        _ => false,
    }
}

fn count_words_naive(corpus: &[u8]) -> usize {
    let mut count = 0;
    let mut previous_whitespace = true;

    for &byte in corpus {
        let current_whitespace = is_whitespace(byte);

        if !current_whitespace && previous_whitespace {
            count += 1;
        }

        previous_whitespace = current_whitespace;
    }

    count
}

fn main() {
    tracing_subscriber::fmt().compact().without_time().init();

    let start = std::time::Instant::now();
    let corpus = generate_corpus();
    info!(elapsed = ?start.elapsed(), "generate_corpus");

    let start = std::time::Instant::now();
    let count = count_words_naive(&corpus);
    info!(elapsed = ?start.elapsed(), words=count, "count_words_naive");
}
