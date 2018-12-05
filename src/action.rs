use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(PartialEq, Eq)]
pub enum Action {
    DARKEN,
    LIGHTEN,
    IDLE
}

impl Distribution<Action> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        match rng.gen_range(0, 3) {
            0 => Action::DARKEN,
            1 => Action::LIGHTEN,
            _ => Action::IDLE
        }
    }
}
