use bitvec::prelude::*;

pub struct Dna {
  pub color: [f32;3],
  pub reaction_time: u16
}

impl Dna {
  fn f8(a: &BitVec, b: &BitVec, i: usize) -> f32 { return Dna::u8(&a,&b,i) as f32 / u8::MAX as f32; }
  // fn u16(a: &BitVec, b: &BitVec, i: usize) -> u16 { return (Dna::u8(a,b,i) as u16) << 8 | Dna::u8(a,b,i+8) as u16; }
  fn u8(a: &BitVec, b: &BitVec, i: usize) -> u8 { return a[i..i+4].load::<u8>() << 4 | b[i..i+4].load::<u8>(); }
  fn b2(a: &BitVec, b: &BitVec, i: usize) -> u8 { return u8::from(a[i]) << 1 | b[i] as u8; }

  pub fn new(a: BitVec, b: BitVec) -> Dna {
    let mut color: [f32;3] = [0.0; 3];
    color[if 0 >= Dna::b2(&a,&b,0) {0} else {1}] = Dna::f8(&a,&b,2);
    color[if 1 >= Dna::b2(&a,&b,0) {1} else {2}] = Dna::f8(&a,&b,10);
    return Dna {
      color,
      reaction_time: 1000u16 + Dna::u8(&a,&b,11) as u16,
    }
  }
}