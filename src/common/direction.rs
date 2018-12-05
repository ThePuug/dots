use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[derive(PartialEq, Eq)]
pub enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        if *self == Direction::NORTH { return Direction::SOUTH; }
        else if *self == Direction::SOUTH { return Direction::NORTH; }
        else if *self == Direction::WEST { return Direction::EAST; }
        else { return Direction::WEST; }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0,4) {
            0 => Direction::NORTH,
            1 => Direction::EAST,
            2 => Direction::SOUTH,
            _ => Direction::WEST
        }
    }
}
