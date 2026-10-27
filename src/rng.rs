//! A small deterministic-friendly random source.
//!
//! Live picks read from the OS CSPRNG (`/dev/urandom`), which keeps dependencies
//! to zero. When a seed is supplied, the same hand-rolled SplitMix64 used by the
//! companion Rust/Go/Zig card-dealer ports is used instead, with rejection
//! sampling so every card is equally likely.

use std::fs::File;
use std::io::Read;

pub struct Random {
    state: Option<u64>,
}

impl Random {
    /// Live randomness from the OS.
    pub fn live() -> Random {
        Random { state: None }
    }

    /// Repeatable randomness from a seed.
    pub fn seeded(seed: u64) -> Random {
        Random { state: Some(seed) }
    }

    fn next_u64(&mut self) -> u64 {
        match self.state {
            Some(mut z) => {
                self.state = Some(z.wrapping_add(0x9E37_79B9_7F4A_7C15));
                z = self.state.unwrap();
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^ (z >> 31)
            }
            None => {
                let mut buf = [0u8; 8];
                let mut f = File::open("/dev/urandom").expect("/dev/urandom unavailable");
                f.read_exact(&mut buf).expect("failed reading /dev/urandom");
                u64::from_ne_bytes(buf)
            }
        }
    }

    /// Uniform value in `0..n` by rejection sampling (no modulo bias).
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "n must be positive");
        let width = n as u64;
        let threshold = u64::MAX - (u64::MAX % width) - 1;
        loop {
            let x = self.next_u64();
            if x <= threshold {
                return (x % width) as usize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn seeded_sequences_are_reproducible() {
        let mut a = Random::seeded(42);
        let mut b = Random::seeded(42);
        let seq_a: Vec<usize> = (0..32).map(|_| a.below(52)).collect();
        let seq_b: Vec<usize> = (0..32).map(|_| b.below(52)).collect();
        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = Random::seeded(1);
        let mut b = Random::seeded(2);
        let seq_a: Vec<usize> = (0..32).map(|_| a.below(52)).collect();
        let seq_b: Vec<usize> = (0..32).map(|_| b.below(52)).collect();
        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn stays_in_range_and_covers_the_deck() {
        let mut r = Random::seeded(7);
        let mut seen = HashSet::new();
        for _ in 0..5_000 {
            let v = r.below(52);
            assert!(v < 52);
            seen.insert(v);
        }
        assert_eq!(seen.len(), 52, "should reach every index given enough draws");
    }

    #[test]
    fn live_source_produces_values_in_range() {
        let mut r = Random::live();
        for _ in 0..200 {
            assert!(r.below(52) < 52);
        }
    }

    #[test]
    fn below_one_is_always_zero() {
        let mut r = Random::seeded(3);
        for _ in 0..10 {
            assert_eq!(r.below(1), 0);
        }
    }
}
