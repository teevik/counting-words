/// Generate a 1GiB string of words with whitespaces
/// From https://github.com/healeycodes/counting-words-at-simd-speed/blob/main/setup_benchmark.c
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

const BYTES_TOTAL: u64 = 1 * 1024 * 1024 * 1024;
const SEED: u64 = 0x243F6A88_85A308D3;
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const WHITESPACE: &[u8] = b" \n\r\t\x0b\x0c";

fn word_len(rng: &mut SmallRng) -> usize {
    loop {
        let b: u8 = rng.random();
        if b < 240 {
            return (b % 30) as usize + 1;
        }
    }
}

fn base62(rng: &mut SmallRng) -> u8 {
    loop {
        let b: u8 = rng.random();
        if b < 248 {
            return ALPHABET[(b % 62) as usize];
        }
    }
}

fn ws_char(rng: &mut SmallRng) -> u8 {
    loop {
        let b: u8 = rng.random();
        if b < 252 {
            return WHITESPACE[(b % 6) as usize];
        }
    }
}

pub fn generate_corpus() -> Vec<u8> {
    let mut buf = Vec::with_capacity(BYTES_TOTAL as usize);
    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut remaining: u64 = BYTES_TOTAL;

    while remaining > 0 {
        if remaining == 1 {
            buf.push(base62(&mut rng));
            remaining -= 1;
        } else if remaining <= 31 {
            let r = (remaining - 1) as usize;
            for _ in 0..r {
                buf.push(base62(&mut rng));
            }
            buf.push(ws_char(&mut rng));
            remaining = 0;
        } else {
            let r = word_len(&mut rng);
            for _ in 0..r {
                buf.push(base62(&mut rng));
            }
            buf.push(ws_char(&mut rng));
            remaining -= (r + 1) as u64;
        }
    }

    assert_eq!(buf.len(), BYTES_TOTAL as usize);

    buf
}
