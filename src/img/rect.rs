use crate::calc;
use crate::Quad;
use crate::INT_MAX;
use std::ops::{Bound, RangeBounds};

type Coord = (u32, u32);

#[derive(Debug, Copy, Clone)]
pub struct Rect(Coord, Coord);

impl Quad for Rect {
    fn width(&self) -> u32 {
        calc!((self.x2()) - (self.x()))
    }

    fn height(&self) -> u32 {
        calc!((self.y2()) - (self.y()))
    }
}

impl Rect {
    pub fn x(&self) -> u32 {
        self.0 .0
    }

    pub fn y(&self) -> u32 {
        self.0 .1
    }

    pub fn x2(&self) -> u32 {
        self.1 .0
    }

    pub fn y2(&self) -> u32 {
        self.1 .1
    }

    fn new(xy: Coord, xy2: Coord) -> Self {
        let (x, y) = (xy.0.min(INT_MAX), xy.0.min(INT_MAX));
        let (x2, y2) = (xy2.0.min(INT_MAX), xy2.1.min(INT_MAX));

        Self((x.min(x2), y.min(y2)), (x.max(x2), y.max(y2)))
    }

    fn from_dimensions(xy: Coord, width: u32, height: u32) -> Self {
        let (x, y) = xy;
        let x2 = calc!(width + x).min(INT_MAX);
        let y2 = calc!(height + y).min(INT_MAX);

        Rect((x, y), (x2, y2))
    }

    pub fn from_range(xx2: impl RangeBounds<u32>, yy2: impl RangeBounds<u32>) -> Rect {
        use self::Bound::{Excluded, Included, Unbounded};

        let point = |bound: Bound<&u32>, default: u32| match bound {
            Included(x) => *x,
            Excluded(x) => calc!(*x - 1),
            Unbounded => default,
        };

        let x = point(xx2.start_bound(), 0);
        let y = point(yy2.start_bound(), 0);
        let x2 = point(xx2.end_bound(), INT_MAX);
        let y2 = point(yy2.end_bound(), INT_MAX);

        Rect((x.min(x2), y.min(y2)), (x.max(x2), y.max(y2)))
    }

    pub fn constrain(&self, width: u32, height: u32) -> Self {
        Rect(
            (width.min(self.x()), height.min(self.y())),
            (width.min(self.x2()), height.min(self.y2())),
        )
    }
}

mod tests {
    use super::{Quad, Rect};
    use crate::INT_MAX;

    #[test]
    fn rect() {
        let bounded = Rect::new((0, 0), (100, 100));
        assert_eq!(bounded.width(), 100);
        assert_eq!(bounded.height(), 100);
        assert_eq!(bounded.area().unwrap(), 10000);

        let inverted = Rect::new((100, 100), (50, 50));
        assert_eq!(inverted.width(), 0);
        assert_eq!(inverted.height(), 0);
    }

    #[test]
    fn from_dimensions() {
        let rect = Rect::from_dimensions((100, 100), 200, 200);
        assert_eq!(rect.width(), 200);
        assert_eq!(rect.height(), 200);
        assert_eq!(rect.x(), 100);
        assert_eq!(rect.y(), 100);
        assert_eq!(rect.x2(), 300);
        assert_eq!(rect.y2(), 300);
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

        let reversed = Rect::from_range(100..=0, 100..=0);
        assert_eq!(reversed.width(), 100);
        assert_eq!(reversed.height(), 100);
        assert_eq!(reversed.x(), 0);
        assert_eq!(reversed.y(), 0);
    }

    #[test]
    fn from_range_outer_limit() {
        let origin = Rect::from_range(0.., 0..).constrain(500, 500);
        assert_eq!(origin.width(), 500);
        assert_eq!(origin.height(), 500);

        let offset = Rect::from_range(100.., 100..).constrain(500, 500);
        assert_eq!(offset.width(), 400);
        assert_eq!(offset.height(), 400);

        let uneven = Rect::new((100, 400), (200, 600)).constrain(500, 500);
        assert_eq!(uneven.width(), 100);
        assert_eq!(uneven.height(), 100);

        let unbounded = Rect::new((100, 600), (700, 800)).constrain(500, 500);
        assert_eq!(unbounded.width(), 400);
        assert_eq!(unbounded.height(), 0);
    }
}
