//! The fuzz baseline's mutator for this crate's untrusted-byte parsers
//! (`SIGNOFF-REPAIR.11.4.7.2.1.1`). The same seeded xorshift as
//! `reasonbraid-extract`'s: a failing round reproduces from its number, and no
//! crate is added for it. Unguided: there is no nightly toolchain and no
//! `cargo-fuzz` in this environment, which the decision record states.

/// A seeded mutator that applies one to four structural edits to a valid seed.
pub(crate) struct Mutator(u64);

impl Mutator {
    pub(crate) fn new(seed: u64) -> Self {
        Mutator(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next() % bound as u64) as usize
        }
    }

    pub(crate) fn mutate(&mut self, seed: &[u8]) -> Vec<u8> {
        let mut bytes = seed.to_vec();
        for _ in 0..=self.below(4) {
            let at = self.below(bytes.len());
            match self.below(6) {
                0 if !bytes.is_empty() => bytes[at] ^= 1 << self.below(8),
                1 if !bytes.is_empty() => bytes[at] = [0x00, 0x7f, 0x80, 0xff][self.below(4)],
                2 => bytes.truncate(at),
                3 => bytes.insert(at, self.next() as u8),
                4 if !bytes.is_empty() => {
                    let end = (at + 1 + self.below(64)).min(bytes.len());
                    let copy = bytes[at..end].to_vec();
                    let to = self.below(bytes.len());
                    bytes.splice(to..to, copy);
                }
                _ if !bytes.is_empty() => {
                    let end = (at + 1 + self.below(16)).min(bytes.len());
                    bytes[at..end].fill(0);
                }
                _ => {}
            }
        }
        bytes
    }
}

/// Rounds per seed: small enough for every test run; `RB_FUZZ_ROUNDS` raises it.
pub(crate) fn rounds() -> usize {
    std::env::var("RB_FUZZ_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

/// Run `parse` on every mutation of every seed, and fail naming the round and
/// the input when one panics.
pub(crate) fn survive<T>(label: &str, seeds: &[Vec<u8>], mut parse: impl FnMut(&[u8]) -> T) {
    let mut mutator = Mutator::new(0x5eed_0bad_c0de_2929);
    for seed in seeds {
        for round in 0..rounds() {
            let input = mutator.mutate(seed);
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parse(&input)));
            assert!(
                outcome.is_ok(),
                "{label} panicked on round {round}: {} bytes, hex {}",
                input.len(),
                input.iter().map(|b| format!("{b:02x}")).collect::<String>()
            );
        }
    }
}
