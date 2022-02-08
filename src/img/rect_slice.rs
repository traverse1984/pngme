use super::img::Img;
use super::rect::Rect;
use crate::{calc, convert, err::*, png, Image, Quad};
use std::ops::{Bound, RangeBounds};

pub struct RectSlice<'a> {
    img: &'a mut Img,
    rect: Rect,
}

impl Quad for RectSlice<'_> {
    fn width(&self) -> u32 {
        self.rect.width()
    }

    fn height(&self) -> u32 {
        self.rect.height()
    }
}

impl Image for RectSlice<'_> {
    fn to_vec(self) -> Vec<u32> {
        self.clone_to_vec()
    }

    fn clone_to_vec(&self) -> Vec<u32> {
        Indexer::from_slice(self.rect())
            .into_iter()
            .map(|idx| self.img.data().get(idx).unwrap())
            .copied()
            .collect()
    }
}

impl<'a> RectSlice<'a> {
    pub fn new(img: &'a mut Img, rect: Rect) -> Self {
        Self { img, rect }
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    fn reslice(
        &mut self,
        xx2: impl RangeBounds<u32>,
        yy2: impl RangeBounds<u32>,
    ) -> PngRes<&mut Self> {
        self.rect = Rect::from_range(xx2, yy2).constrain(self.img.width(), self.img.height());
        Ok(self)
    }

    pub fn clamp(&mut self, width: u32, height: u32) -> &mut Self {
        //self.rect.constrain(width, height);
        self
    }

    pub fn fill(&mut self, col: u32) -> &mut Self {
        self.iter_mut().for_each(|px| *px = col);
        self
    }

    pub fn copy_from(&mut self, data: &[u32]) -> &mut Self {
        self.iter_mut()
            .zip(data.iter())
            .for_each(|(curr, new)| *curr = *new);
        self
    }

    pub fn to_img(self) -> Img {
        Img::from_vec(self.rect.width(), self.rect.height(), self.to_vec())
    }

    pub fn iter(&self) -> Iter<'a, '_> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'a, '_> {
        IterMut::new(self)
    }
}

struct Indexer {
    rect: Rect,
    offset: usize,
    idx: usize,
}

impl Indexer {
    pub fn from_slice(rect: &Rect) -> Self {
        let (y, width) = (
            convert!(ex usize; rect.y()),
            convert!(ex usize; rect.width()),
        );

        let offset = calc!(y * width);

        Self {
            offset: calc!(y * width),
            rect: *rect,
            idx: convert!(ex usize; rect.x()),
        }
    }
}

impl Iterator for Indexer {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let width = convert!(ex usize; self.rect.width());
        if self.idx >= width {
            self.offset += width;
            if self.offset / width > convert!(ex usize; self.rect.height()) {
                return None;
            }
            self.idx = convert!(ex usize; self.rect.x());
        }

        let index = self.offset + self.idx;
        self.idx += 1;
        Some(index)
    }
}

pub struct Iter<'a, 'b> {
    slice: &'b RectSlice<'a>,
    idx: Indexer,
}

impl<'a, 'b> Iter<'a, 'b> {
    pub fn new(slice: &'b RectSlice<'a>) -> Self {
        Self {
            slice,
            idx: Indexer::from_slice(slice.rect()).into_iter(),
        }
    }
}

impl<'a, 'b> Iterator for Iter<'a, 'b> {
    type Item = &'b u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx
            .next()
            .map_or(None, |idx| self.slice.img.data().get(idx))
    }
}

pub struct IterMut<'a, 'b> {
    slice: &'b mut RectSlice<'a>,
    idx: Indexer,
}

impl<'a, 'b> IterMut<'a, 'b> {
    pub fn new(slice: &'b mut RectSlice<'a>) -> Self {
        let idx = Indexer::from_slice(slice.rect()).into_iter();
        Self { slice, idx }
    }
}

impl<'a, 'b> Iterator for IterMut<'a, 'b> {
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
