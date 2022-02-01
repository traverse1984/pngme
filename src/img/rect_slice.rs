use super::image::Image;
use super::image_data::*;
use super::rect::Rect;
use crate::png;
use std::ops::{Bound, RangeBounds};

pub struct RectSlice<'a> {
    img: &'a mut Image,
    rect: Rect,
}

impl<'a> Quad for RectSlice<'a> {
    fn width(&self) -> u32 {
        self.rect.width()
    }

    fn height(&self) -> u32 {
        self.rect.height()
    }
}

//impl<'a> ImageData for RectSlice<'a> {}

impl<'a> RectSlice<'a> {
    pub fn clamp(&mut self, width: u32, height: u32) -> &mut Self {
        self.rect.constrain(width, height);
        self
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    pub fn new(img: &'a mut Image, rect: Rect) -> Self {
        Self { img, rect }
    }

    pub fn slice(&mut self, xx2: impl RangeBounds<u32>, yy2: impl RangeBounds<u32>) -> &mut Self {
        self.rect = Rect::from_range(xx2, yy2);
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
            .chunks(png::expect_usize(self.img.width()))
            .map(|row| Vec::from(row))
            .collect()
    }
}

struct SliceIndexIter {
    rect: Rect,
    offset: usize,
    idx: usize,
}

impl SliceIndexIter {
    pub fn from_slice(rect: &Rect) -> Self {
        let offset =
            usize::checked_mul(png::expect_usize(rect.y()), png::expect_usize(rect.width()))
                .expect("Rect was not within bounds.");

        Self {
            offset,
            rect: *rect,
            idx: png::expect_usize(rect.x()),
        }
    }
}

impl Iterator for SliceIndexIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        // if self.idx > self.x2 {
        //     self.offset += self.width;
        //     if self.offset / self.width > self.y2 {
        //         return None;
        //     }
        //     self.idx = self.x;
        // }

        // let index = self.offset + self.idx;
        // self.idx += 1;
        // Some(index)
        None
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
            idx: SliceIndexIter::from_slice(slice.rect()).into_iter(),
        }
    }
}

impl<'a, 'b> Iterator for SliceIter<'a, 'b> {
    type Item = &'b u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx
            .next()
            .map_or(None, |idx| self.slice.img.data().get(idx))
    }
}

pub struct RectSliceIterMut<'a, 'b> {
    slice: &'b mut RectSlice<'a>,
    idx: SliceIndexIter,
}

impl<'a, 'b> RectSliceIterMut<'a, 'b> {
    pub fn new(slice: &'b mut RectSlice<'a>) -> Self {
        let idx = SliceIndexIter::from_slice(slice.rect()).into_iter();
        Self { slice, idx }
    }
}

impl<'a, 'b> Iterator for RectSliceIterMut<'a, 'b> {
    type Item = &'b mut u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx.next().map_or(None, |idx| {
            self.slice
                .img
                .data_mut()
                .get_mut(idx)
                .map(|px| unsafe { &mut *(px as *mut u32) })
        })
    }
}
