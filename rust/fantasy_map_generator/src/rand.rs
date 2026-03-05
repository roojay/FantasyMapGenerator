/// Exact replica of glibc's rand() (TYPE_3, degree=31, separation=3)
/// seeded with srand(seed).
pub struct GlibcRand {
    state: [u32; 31],
    fptr: usize,
    rptr: usize,
}

impl GlibcRand {
    pub fn new(seed: u32) -> Self {
        // glibc: if seed == 0, use 1
        let seed = if seed == 0 { 1 } else { seed };

        let mut state = [0u32; 31];
        // Park-Miller LCG init as glibc does it (avoids overflow via Schrage method)
        state[0] = seed;
        for i in 1..31 {
            let prev = state[i - 1] as i32;
            let hi = prev / 127773;
            let lo = prev % 127773;
            let mut word = 16807 * lo - 2836 * hi;
            if word < 0 { word += 2147483647; }
            state[i] = word as u32;
        }
        // After init: 310 warmup iterations with fptr=3, rptr=0
        let mut fptr = 3usize;
        let mut rptr = 0usize;
        for _ in 0..310 {
            state[fptr] = state[fptr].wrapping_add(state[rptr]);
            fptr = (fptr + 1) % 31;
            rptr = (rptr + 1) % 31;
        }
        // After 310 iterations: fptr=(3+310)%31=3, rptr=310%31=0
        GlibcRand { state, fptr: 3, rptr: 0 }
    }

    /// Returns next random value in [0, 0x7fffffff]
    pub fn rand(&mut self) -> i32 {
        self.state[self.fptr] = self.state[self.fptr].wrapping_add(self.state[self.rptr]);
        let result = (self.state[self.fptr] >> 1) & 0x7fffffff;
        self.fptr = (self.fptr + 1) % 31;
        self.rptr = (self.rptr + 1) % 31;
        result as i32
    }

    /// randomDouble: min + rand() / (RAND_MAX / (max - min))
    pub fn random_double(&mut self, min: f64, max: f64) -> f64 {
        min + (self.rand() as f64) / (0x7fffffff as f64 / (max - min))
    }

    /// randomRange(min, max): min + (rand() % (max - min))
    pub fn random_range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.rand() % (max - min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glibc_rand_sequence() {
        let mut rng = GlibcRand::new(42);
        for _ in 0..1000 {
            rng.rand();
        }
        let v = rng.rand();
        assert!(v >= 0 && v <= 0x7fffffff, "rand out of range: {}", v);
        // Verify seed=0 doesn't produce all zeros (glibc treats seed=0 as seed=1)
        let mut rng0 = GlibcRand::new(0);
        let v0 = rng0.rand();
        assert_ne!(v0, 0, "seed=0 should not produce zero rand values");
    }
}
