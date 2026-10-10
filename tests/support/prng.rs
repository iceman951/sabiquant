//! Deterministic pseudo-random numbers for property tests (F-CORE-05;
//! TESTING_STRATEGY §5).
//!
//! The generator is SplitMix64, from G. L. Steele Jr., D. Lea and
//! C. H. Flood, "Fast splittable pseudorandom number generators", OOPSLA 2014,
//! implemented from the public-domain reference implementation by S. Vigna
//! (<https://prng.di.unimi.it/splitmix64.c>). It uses only wrapping 64-bit
//! integer arithmetic, so it produces the same sequence on every platform.
//! It is a test tool, not a production or cryptographic generator.
//!
//! Property tests run through [`check`], which gives every case its own
//! recorded seed. A failure report names that seed, so the failing case can be
//! replayed alone and turned into a regression test. There is no automatic
//! shrinking (TESTING_STRATEGY §5).

/// The SplitMix64 generator.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// The increment of the state: 2^64 divided by the golden ratio, made odd.
    const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

    /// Creates a generator from `seed`. Every seed is valid, including 0.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns the next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(Self::GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a uniform value in `[0, 1)` with 53 random bits, so every
    /// result is a multiple of 2^-53.
    pub fn next_f64(&mut self) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a 53-bit integer is exactly representable in f64"
        )]
        let numerator = (self.next_u64() >> 11) as f64;
        numerator * (1.0 / 9_007_199_254_740_992.0) // 2^-53, exact
    }

    /// Returns a uniform value in `[0, bound)`, without modulo bias.
    ///
    /// # Panics
    ///
    /// If `bound` is 0.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "the bound must be positive");
        // Reject the lowest 2^64 mod `bound` values, so that every residue
        // has the same number of preimages.
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let x = self.next_u64();
            if x >= threshold {
                return x % bound;
            }
        }
    }

    /// Returns a uniform value in `[low, high]`, both ends included.
    ///
    /// # Panics
    ///
    /// If `low > high`.
    pub fn range_i64(&mut self, low: i64, high: i64) -> i64 {
        assert!(low <= high, "empty range [{low}, {high}]");
        let span = i128::from(high) - i128::from(low) + 1;
        let offset = match u64::try_from(span) {
            Ok(span) => self.below(span),
            // The full range of i64 holds 2^64 values: every bit pattern.
            Err(_) => self.next_u64(),
        };
        i64::try_from(i128::from(low) + i128::from(offset)).expect("the result lies in the range")
    }

    /// Returns an arbitrary `f64` bit pattern: a normal or subnormal number, a
    /// zero, an infinity or a NaN.
    pub fn any_f64(&mut self) -> f64 {
        f64::from_bits(self.next_u64())
    }

    /// Returns a uniformly chosen element of `items`.
    ///
    /// # Panics
    ///
    /// If `items` is empty.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        assert!(!items.is_empty(), "cannot choose from an empty slice");
        let count = u64::try_from(items.len()).expect("a slice length fits in u64");
        let index = usize::try_from(self.below(count)).expect("the index is below the length");
        &items[index]
    }
}

/// Returns the seed of case `case` of a property run started from `seed`.
///
/// It is output number `case` (counting from 0) of `SplitMix64::new(seed)`,
/// computed directly: the state after `case + 1` steps is
/// `seed + (case + 1) * GAMMA`.
pub fn case_seed(seed: u64, case: u64) -> u64 {
    SplitMix64::new(seed.wrapping_add(case.wrapping_mul(SplitMix64::GAMMA))).next_u64()
}

/// Runs `property` on `cases` cases. Case `k` gets a generator seeded with
/// `case_seed(seed, k)`.
///
/// The property returns `Err(message)` to report a failure, which panics with
/// the property name, the case number, the run seed and the case seed.
/// `SplitMix64::new(case seed)` then replays the failing case on its own.
#[track_caller]
pub fn check<F>(name: &str, seed: u64, cases: u64, mut property: F)
where
    F: FnMut(&mut SplitMix64) -> Result<(), String>,
{
    for case in 0..cases {
        let case_seed = case_seed(seed, case);
        let mut rng = SplitMix64::new(case_seed);
        if let Err(message) = property(&mut rng) {
            panic!(
                "property `{name}` failed at case {case} of {cases} \
                 (run seed {seed:#018x}, case seed {case_seed:#018x}): {message}"
            );
        }
    }
}
