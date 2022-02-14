use super::{img::Img, rect::Rect};
use crate::{calc, convert, err::*, Image, Quad};
use std::ops::RangeBounds;

#[derive(Debug)]
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
        self.iter().copied().collect()
    }
}

impl<'a> RectSlice<'a> {
    pub fn new(img: &'a mut Img, rect: Rect) -> Self {
        Self { img, rect }
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    fn set_rect(&mut self, rect: Rect) -> &mut Self {
        self.rect = rect.constrain(self.img.width(), self.img.height());
        self
    }

    pub fn reslice(&mut self, xx2: impl RangeBounds<u32>, yy2: impl RangeBounds<u32>) -> &mut Self {
        self.set_rect(Rect::from_range(xx2, yy2))
    }

    pub fn xywh(&mut self, xy: (u32, u32), width: u32, height: u32) -> &mut Self {
        self.set_rect(Rect::from_dimensions(xy, width, height))
    }

    pub fn pos(&mut self, x: u32, y: u32) -> &mut Self {
        self.set_rect(self.rect.pos(x, y))
    }

    pub fn fill(&mut self, col: u32) -> &mut Self {
        self.iter_mut().for_each(|px| *px = col);
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.iter_mut().for_each(|px| *px = 0);
        self
    }

    pub fn copy_from(&mut self, data: &[u32]) -> &mut Self {
        self.iter_mut()
            .zip(data.iter())
            .for_each(|(curr, new)| *curr = *new);
        self
    }

    pub fn copy_each(&mut self, x: u32, y: u32, filter: impl Fn(&u32, &u32) -> u32) -> &mut Self {
        let from = self.rect;
        let to = self
            .rect
            .pos(x, y)
            .constrain(self.img.width(), self.img.height());

        if from.area() > to.area() {
            self.rect = self.rect.constrain(to.width(), to.height());
        }

        let from_data = self.clone_to_vec();
        self.pos(x, y)
            .iter_mut()
            .zip(from_data)
            .for_each(|(px, px_from)| *px = filter(&px_from, px));

        self.rect = from;
        self
    }

    pub fn copy(&mut self, x: u32, y: u32) -> &mut Self {
        self.copy_each(x, y, |from, _| *from)
    }

    pub fn clone_to_img(&self) -> Img {
        Img::from_vec(self.rect.width(), self.rect.height(), self.clone_to_vec())
    }

    pub fn iter(&self) -> Iter<'a, '_> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'a, '_> {
        IterMut::new(self)
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
            idx: Indexer::from_slice(slice.rect(), slice.img.width()).into_iter(),
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
        let idx = Indexer::from_slice(slice.rect(), slice.img.width()).into_iter();
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

struct Indexer {
    rect: Rect,
    offset: usize,
    idx: u32,
    img_width: u32,
}

impl Indexer {
    pub fn from_slice(rect: &Rect, img_width: u32) -> Self {
        let (ypos, width) = convert!(ex usize; rect.y(), img_width);
        Self {
            img_width,
            offset: calc!(ypos * width),
            rect: *rect,
            idx: rect.x(),
        }
    }
}

impl Iterator for Indexer {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let (img_width, y2) = convert!(ex usize; self.img_width, self.rect.y2());

        if self.idx >= self.rect.x2() {
            self.offset += img_width;
            if self.offset / img_width >= y2 {
                return None;
            }
            self.idx = self.rect.x();
        }

        let index = self.offset + convert!(ex usize; self.idx);
        self.idx += 1;
        Some(index)
    }
}
