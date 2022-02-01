use crate::err::PngRes;
use crate::png;

pub trait Quad {
    fn height(&self) -> u32;
    fn width(&self) -> u32;

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn area(&self) -> PngRes<usize> {
        png::area(self.width(), self.height())
    }
}

pub trait ImageData: Quad {
    fn to_vec(self) -> Vec<u32>;
    fn clone_to_vec(&self) -> Vec<u32>;
    fn to_bytes(&self) -> Vec<u8>;

    fn data(&self) -> &Vec<u32>;
    fn data_mut(&mut self) -> &mut Vec<u32>;
}
