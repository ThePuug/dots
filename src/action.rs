use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[derive(PartialEq, Eq)]
pub enum Action {
    DIGEST,
    SEED,
    IDLE
}

impl Distribution<Action> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        match rng.gen_range(0..3) {
            0 => Action::DIGEST,
            1 => Action::SEED,
            _ => Action::IDLE,
        }
    }
}
