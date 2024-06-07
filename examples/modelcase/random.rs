use rand::SeedableRng;
use street_engine::transport::traits::RandomF64Provider;

pub struct RandomF64<R> {
    rng: R,
}

impl<R: rand::Rng> RandomF64Provider for RandomF64<R> {
    fn gen_f64(&mut self) -> f64 {
        self.rng.gen()
    }
}

impl RandomF64<rand::rngs::StdRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(0),
        }
    }
}
