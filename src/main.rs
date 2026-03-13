#![allow(unsafe_op_in_unsafe_fn)]

use crate::generate::generate_corpus;
use std::arch::aarch64::*;

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

unsafe fn count_words_simd(corpus: &[u8]) -> usize {
    let mut words = 0;

    // Initialize the previous whitespace mask to be all `true`
    let mut prev_whitespace = vdupq_n_u8(0b1111_1111);

    // Broadcasted filters for each whitespace character
    let whitespace_filters = [
        vdupq_n_u8(b' '),
        vdupq_n_u8(b'\n'),
        vdupq_n_u8(b'\r'),
        vdupq_n_u8(b'\t'),
        vdupq_n_u8(b'\x0b'),
        vdupq_n_u8(b'\x0c'),
    ];

    // 128 bytes at a time
    let mut chunks = corpus.chunks_exact(16);

    for chunk in &mut chunks {
        // Load into vector
        let bytes = vld1q_u8(chunk.as_ptr());

        // Compare each byte against each whitespace filter
        let whitespace_masks = whitespace_filters.map(|mask| vceqq_u8(bytes, mask));

        // Combine all masks into a single mask
        let whitespace = whitespace_masks
            .into_iter()
            .reduce(|acc, mask| vorrq_u8(acc, mask))
            .unwrap_unchecked();

        // Shift the previous whitespace mask to the right and combine with the current whitespace mask
        let prev_whitespace_shifted = vextq_u8(prev_whitespace, whitespace, 15);

        // Find non-whitespace characters
        let non_whitespace = vmvnq_u8(whitespace);
        // Combine with shifted previous whitespace mask to find word starts
        let start_mask = vandq_u8(non_whitespace, prev_whitespace_shifted);

        // Count the number of word starts
        let ones = vshrq_n_u8(start_mask, 7);

        // Sum number of word starts
        words += vaddvq_u8(ones) as usize;

        prev_whitespace = whitespace;
    }

    // Scalar solution for the remainder
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let mut prev_ws = vgetq_lane_u8(prev_whitespace, 15) != 0;

        for &byte in remainder {
            let current_ws = is_whitespace(byte);
            if !current_ws && prev_ws {
                words += 1;
            }
            prev_ws = current_ws;
        }
    }

    words
}

fn main() {
    let start = std::time::Instant::now();
    let corpus = generate_corpus();
    println!("Generated 1GiB of text in {:?}", start.elapsed());

    // Assert 16-byte alignment
    assert_eq!(corpus.as_ptr() as usize % 16, 0);

    let start = std::time::Instant::now();
    let count_naive = count_words_naive(&corpus);
    println!("count_words_naive: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let count_simd = unsafe { count_words_simd(&corpus) };
    println!("count_words_simd: {:?}", start.elapsed());

    dbg!(count_naive, count_simd);
    assert_eq!(count_naive, count_simd);
}
