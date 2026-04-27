//! Simple random number generator (XorShift128)
//! Replaces `rand` and `rand_chacha` crates

#[derive(Clone, Debug)]
pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    /// Create a new RNG with a seed
    pub fn new(seed: u64) -> Self {
        // Initialize state using seed
        let state = [
            seed.wrapping_add(0x1234567890abcdef),
            seed.wrapping_add(0xfedcba0987654321),
            seed.wrapping_add(0xabcdef1234567890),
            seed.wrapping_add(0x0987654321fedcba),
        ];
        
        // Warm up the generator
        let mut rng = Self { state };
        for _ in 0..10 {
            rng.next();
        }
        rng
    }

    /// Generate next random u64
    #[inline]
    pub fn next(&mut self) -> u64 {
        let t = self.state[0] ^ (self.state[0] << 11);
        self.state[0] = self.state[1];
        self.state[1] = self.state[2];
        self.state[2] = self.state[3];
        self.state[3] = self.state[3] ^ (self.state[3] >> 19) ^ t ^ (t >> 8);
        self.state[3]
    }

    /// Generate random u32
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next() >> 32) as u32
    }

    /// Generate random f32 in range [0.0, 1.0)
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        ((self.next() >> 12) as f32) / (1u64 << 52) as f32
    }

    /// Generate random f64 in range [0.0, 1.0)
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        ((self.next() >> 12) as f64) / (1u64 << 52) as f64
    }

    /// Generate random integer in range [min, max)
    #[inline]
    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u32() % (max - min))
    }

    /// Generate random float in range [min, max)
    #[inline]
    pub fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(12345)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);
        
        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_rng_range() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let val = rng.next_range(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_rng_float_range() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let val = rng.next_f32_range(0.0, 1.0);
            assert!(val >= 0.0 && val < 1.0);
        }
    }
}
