use core::ops::Range;

pub trait Test<const B: u8, const W: u8> {
    const BIT_DEPTH: u8;
    const FILTER_METHOD: u8 = 0;
    const INTERLACE_METHOD: u8 = 0;
}

pub struct Hi {}

impl Test<0, 8> for Hi {
    const BIT_DEPTH: u8 = 16;
}
