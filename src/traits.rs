use crate::{area, calc, convert, err::*};

pub trait Color: Sized {
    fn _color_value(&self) -> Self;
}

impl Color for u32 {
    fn _color_value(&self) -> Self {
        *self
    }
}

pub trait Quad {
    fn height(&self) -> u32;
    fn width(&self) -> u32;

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn area(&self) -> usize {
        area!(self.width(), self.height())
    }
}

pub trait Image: Quad {
    fn to_vec(self) -> Vec<u32>;
    fn clone_to_vec(&self) -> Vec<u32>;
}
