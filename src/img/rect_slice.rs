use super::ImageData;
use std::ops::{Bound, RangeBounds};

pub struct RectSlice<'a> {
    img: &'a mut ImageData,
    rect: RectCoord,
}

type RectCoord = (usize, usize, usize, usize);

impl<'a> RectSlice<'a> {
    pub fn width(&self) -> usize {
        let (x, _, x2, _) = self.rect;
        x2 - x + 1
    }

    pub fn height(&self) -> usize {
        let (_, y, _, y2) = self.rect;
        y2 - y + 1
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    pub fn clamp(&mut self, width: usize, height: usize) -> &mut Self {
        let wdiff = usize::saturating_sub(self.width(), width);
        let hdiff = usize::saturating_sub(self.height(), height);

        if wdiff > 0 {
            let (x, _, x2, _) = &mut self.rect;
            *x2 = usize::max(*x, usize::saturating_sub(*x2, wdiff));
        }

        if hdiff > 0 {
            let (_, y, _, y2) = &mut self.rect;
            *y2 = usize::max(*y, usize::saturating_sub(*y2, hdiff));
        }

        self
    }

    pub fn rect(&self) -> RectCoord {
        self.rect
    }

    pub fn new(img: &'a mut ImageData) -> Self {
        Self {
            img,
            rect: (0, 0, 0, 0),
        }
    }

    fn range_to_rect(
        &self,
        xx2: impl RangeBounds<usize>,
        yy2: impl RangeBounds<usize>,
    ) -> RectCoord {
        use self::Bound::{Excluded, Included, Unbounded};

        let to_index = |bound: Bound<&usize>, auto: usize, max: usize| {
            let pixel = match bound {
                Included(x) => *x,
                Excluded(x) => usize::saturating_sub(*x, 1),
                Unbounded => auto,
            };
            usize::min(pixel, max)
        };

        let (width, height) = (self.img.width() - 1, self.img.height() - 1);

        let x = to_index(xx2.start_bound(), 0, width);
        let y = to_index(yy2.start_bound(), 0, height);
        let x2 = to_index(xx2.end_bound(), width, width);
        let y2 = to_index(yy2.end_bound(), height, height);

        (
            usize::min(x, x2),
            usize::min(y, y2),
            usize::max(x, x2),
            usize::max(y, y2),
        )
    }

    pub fn slice(
        &mut self,
        xx2: impl RangeBounds<usize>,
        yy2: impl RangeBounds<usize>,
    ) -> &mut Self {
        self.rect = self.range_to_rect(xx2, yy2);
        self
    }

    pub fn fill(&mut self, col: u32) -> &mut Self {
        self.iter_mut().for_each(|px| *px = col);
        self
    }

    pub fn copy_from_vec(&mut self, data: Vec<u32>) -> &mut Self {
        self.iter_mut()
            .zip(data.iter())
            .for_each(|(curr, new)| *curr = *new);
        self
    }

    pub fn iter_mut(&mut self) -> RectSliceIterMut<'a, '_> {
        RectSliceIterMut::new(self)
    }

    pub fn iter(&self) -> SliceIter<'a, '_> {
        SliceIter::new(self)
    }

    pub fn to_vec(&self) -> Vec<u32> {
        self.iter().copied().collect()
    }

    pub fn to_vec_2d(&self) -> Vec<Vec<u32>> {
        self.to_vec()
            .chunks(self.img.width())
            .map(|row| Vec::from(row))
            .collect()
    }
}

struct SliceIndexIter {
    width: usize,
    offset: usize,
    idx: usize,
    x: usize,
    x2: usize,
    y2: usize,
}

impl SliceIndexIter {
    pub fn from_slice(slice: &RectSlice) -> Self {
        let (x, y, x2, y2) = slice.rect;
        Self {
            width: slice.img.width(),
            offset: y * slice.img.width(),
            idx: x,
            x,
            x2,
            y2,
        }
    }
}

impl Iterator for SliceIndexIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.x2 {
            self.offset += self.width;
            if self.offset / self.width > self.y2 {
                return None;
            }
            self.idx = self.x;
        }

        let index = self.offset + self.idx;
        self.idx += 1;
        Some(index)
    }
}

pub struct SliceIter<'a, 'b> {
    slice: &'b RectSlice<'a>,
    idx: SliceIndexIter,
}

impl<'a, 'b> SliceIter<'a, 'b> {
    pub fn new(slice: &'b RectSlice<'a>) -> Self {
        Self {
            slice,
            idx: SliceIndexIter::from_slice(slice).into_iter(),
        }
    }
}

impl<'a, 'b> Iterator for SliceIter<'a, 'b> {
    type Item = &'b u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx
            .next()
            .map_or(None, |idx| self.slice.img.data.get(idx))
    }
}

pub struct RectSliceIterMut<'a, 'b> {
    slice: &'b mut RectSlice<'a>,
    idx: SliceIndexIter,
}

impl<'a, 'b> RectSliceIterMut<'a, 'b> {
    pub fn new(slice: &'b mut RectSlice<'a>) -> Self {
        let idx = SliceIndexIter::from_slice(slice).into_iter();
        Self { slice, idx }
    }
}

impl<'a, 'b> Iterator for RectSliceIterMut<'a, 'b> {
    type Item = &'b mut u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx.next().map_or(None, |idx| {
            self.slice
                .img
                .data
                .get_mut(idx)
                .map(|px| unsafe { &mut *(px as *mut u32) })
        })
    }
}
