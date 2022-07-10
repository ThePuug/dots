use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[derive(PartialEq, Eq)]
pub enum Direction {
    NORTH,
    NORTHEAST,
    EAST,
    SOUTHEAST,
    SOUTH,
    SOUTHWEST,
    WEST,
    NORTHWEST,
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
        match rng.gen_range(0..8) {
            0 => Direction::NORTH,
            1 => Direction::NORTHEAST,
            2 => Direction::EAST,
            3 => Direction::SOUTHEAST,
            4 => Direction::SOUTH,
            5 => Direction::SOUTHWEST,
            6 => Direction::WEST,
            _ => Direction::NORTHWEST,
        }
    }
}
