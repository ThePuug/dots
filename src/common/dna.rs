use bitvec::prelude::*;
use rand::prelude::*;

#[derive(Clone)]
pub struct Dna {
    pub seq: [u8; 8],
}

impl Dna {
    fn f(&self, n: usize, i: usize) -> f32 {
        return f32::try_from(i16::try_from(self.u(n, i)).unwrap()).unwrap()
            / f32::try_from(2i16.pow(u32::try_from(n).unwrap())).unwrap();
    }
    fn u(&self, n: usize, i: usize) -> u32 {
        return self.seq.view_bits::<Lsb0>()[i..i + n].load::<u32>();
    }

    pub fn color(&self) -> [f32; 3] {
        return [self.f(8, 0), self.f(8, 8), self.f(8, 16)];
    }
    pub fn reaction_time(&self) -> u64 {
        return self.u(8, 24).into();
    }

    pub fn combine(&mut self, other: [u8; 8]) {
        let mask: [u8; 8] = thread_rng().gen();
        for (i, it) in mask.into_iter().enumerate() {
            self.seq[i] = (it & self.seq[i]) | (!it & other[i]);
        }
    }

    pub fn new(seq: [u8; 8]) -> Dna {
        return Dna { seq };
    }
}
