use super::image_data::Quad;
use crate::err::*;
use crate::png;
use crate::INT_MAX;
use std::ops::{Bound, RangeBounds};

#[derive(Debug, Copy, Clone)]
pub struct Rect(u32, u32, u32, u32);

impl Quad for Rect {
    fn width(&self) -> u32 {
        self.2
    }

    fn height(&self) -> u32 {
        self.3
    }
}

impl Rect {
    pub fn x(&self) -> u32 {
        self.0
    }

    pub fn y(&self) -> u32 {
        self.1
    }

    pub fn expect_bounded_area(width: u32, height: u32) {
        if png::area(width, height).is_err() {
            panic!(
                "Area of rect {}x{} is too large to be handled by this system.",
                width, height
            );
        }
    }

    pub fn expect_bounded_pos(x: u32, y: u32) {
        if Self::pos_bounded(x, y).is_err() {
            panic!("Rect co-ords ({},{}) are out of bounds.", x, y);
        }
    }

    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self::expect_bounded_pos(x, y);

        let mut rect = Self(x, y, width, height);

        Self::expect_bounded_area(width, height);

        // Need to add overflow feature
        rect
    }

    pub fn constrain(&mut self, max_width: u32, max_height: u32) {
        let Rect(.., width, height) = self;

        *width = png::clamp_u32(u32::min(*width, max_width));
        *height = png::clamp_u32(u32::min(*height, max_height));
    }

    pub fn pos_bounded(x: u32, y: u32) -> PngRes {
        PngErr::not_or(x > INT_MAX, PngErr::XOverflow)?;
        PngErr::not_or(y > INT_MAX, PngErr::YOverflow)
    }

    pub fn dimensions_bounded(width: u32, height: u32) -> PngRes {
        PngErr::not_or(width == 0, PngErr::ZeroWidth)?;
        PngErr::not_or(height == 0, PngErr::ZeroHeight)?;
        Self::pos_bounded(width, height)?;
        png::area(width, height)?;
        Ok(())
    }

    pub fn from_range(xx2: impl RangeBounds<u32>, yy2: impl RangeBounds<u32>) -> Self {
        use self::Bound::{Excluded, Included, Unbounded};

        let point = |bound: Bound<&u32>, auto: u32| match bound {
            Included(x) => *x,
            Excluded(x) => u32::saturating_sub(*x, 1),
            Unbounded => auto,
        };

        let x = point(xx2.start_bound(), 0); // 1..11 -> x=1, 10
        let y = point(yy2.start_bound(), 0);
        let x2 = point(xx2.end_bound(), INT_MAX);
        let y2 = point(yy2.end_bound(), INT_MAX);

        let (x, x2) = (u32::min(x, x2), u32::max(x, x2));
        let (y, y2) = (u32::min(y, y2), u32::max(y, y2));

        Self::new(x, y, x2 - x, y2 - y)
    }
}

impl From<(u32, u32, u32, u32)> for Rect {
    fn from((x, y, width, height): (u32, u32, u32, u32)) -> Self {
        Self::new(x, y, width, height)
    }
}

mod tests {
    use super::{Quad, Rect};
    use crate::INT_MAX;

    #[test]
    fn rect() {
        let one = Rect::new(0, 0, 100, 100);

        assert_eq!(one.width(), 100);
        assert_eq!(one.height(), 100);
        assert_eq!(one.area().unwrap(), 10000);
    }

    #[test]
    fn from_range() {
        let excluded = Rect::from_range(0..100, 0..100);
        assert_eq!(excluded.width(), 99);
        assert_eq!(excluded.height(), 99);

        let included = Rect::from_range(10..=110, 10..=110);
        assert_eq!(included.width(), 100);
        assert_eq!(included.height(), 100);

        let left_ub = Rect::from_range(..100, ..100);
        assert_eq!(left_ub.width(), 99);
        assert_eq!(left_ub.height(), 99);

        let right_ub = Rect::from_range(10.., 10..);
        assert_eq!(right_ub.width(), INT_MAX - 10);
        assert_eq!(right_ub.height(), INT_MAX - 10);
    }
}
