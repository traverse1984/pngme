use crate::chunk::{self, Chunk};
use crate::png::Png;
use crate::png::PngError::{self, *};
use std::fmt;
use std::mem;
use std::ops::RangeBounds;

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_be_bytes([r, g, b, a])
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    rgba(r, g, b, 255)
}

pub fn hex(val: u32) -> u32 {
    let [r_, rg, gb, ba] = val.to_be_bytes();
    if r_ > 0 {
        rgba(r_, rg, gb, ba)
    } else {
        rgb(rg, gb, ba)
    }
}

pub fn hexa(val: u32) -> u32 {
    val
}

pub struct ImageData {
    width: usize,
    height: usize,
    pub data: Vec<u32>,
    filter: Vec<u8>,
}

impl ImageData {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn new(width: usize, height: usize) -> Result<ImageData, PngError> {
        Self::new_bg(width, height, 0)
    }

    pub fn new_bg(width: usize, height: usize, bg: u32) -> Result<ImageData, PngError> {
        Self::checked_dimensions(width, height)?;
        Ok(Self {
            width,
            height,
            data: vec![bg; width * height],
            filter: vec![0; height],
        })
    }

    pub fn slice(
        &mut self,
        xx2: impl RangeBounds<usize>,
        yy2: impl RangeBounds<usize>,
    ) -> RectSlice<'_> {
        let mut rect = RectSlice::new(self);
        rect.slice(xx2, yy2);
        rect
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.data
            .chunks(self.width)
            .map(|chunk| chunk.iter().flat_map(|px| px.to_be_bytes()))
            .zip(self.filter.iter())
            .flat_map(|(row, ft)| ft.to_be_bytes().into_iter().chain(row))
            .collect()
    }

    fn checked_dimensions(width: usize, height: usize) -> Result<(), PngError> {
        PngError::is(width == 0, ZeroWidth)?;
        PngError::is(height == 0, ZeroHeight)?;
        PngError::is(width > Chunk::INT_MAX, WidthOverflow)?;
        PngError::is(height > Chunk::INT_MAX, HeightOverflow)
    }

    fn checked(
        width: usize,
        height: usize,
        data: &Vec<u32>,
        filter: &Vec<u8>,
    ) -> Result<(), PngError> {
        Self::checked_dimensions(width, height)?;
        PngError::is(data.len() != width * height, DataLengthMismatch)?;
        PngError::is(filter.len() != height, FilterLengthMismatch)
    }

    fn from_parts(
        width: usize,
        height: usize,
        data: Vec<u32>,
        filter: Vec<u8>,
    ) -> Result<ImageData, PngError> {
        Self::checked(width, height, &data, &filter)?;

        Ok(Self {
            width,
            height,
            data,
            filter,
        })
    }

    pub fn from_vec_2d(data: Vec<Vec<u32>>) -> Result<ImageData, PngError> {
        let height = data.len();
        let width = data.get(0).ok_or_else(|| ZeroWidth)?.len();

        if data.iter().any(|row| row.len() != width) {
            return Err(WidthMismatch);
        }

        Self::from_parts(
            width,
            height,
            data.into_iter().flatten().collect(),
            vec![0; height],
        )
    }

    pub fn copy(
        &mut self,
        (xx2, yy2): (impl RangeBounds<usize>, impl RangeBounds<usize>),
        (xto, yto): (usize, usize),
    ) {
        self.copy_filter((xx2, yy2), (xto, yto), |x| *x);
        //let slice = self.slice(xx2, yy2).fill(0);
    }

    pub fn copy_filter(
        &mut self,
        (xx2, yy2): (impl RangeBounds<usize>, impl RangeBounds<usize>),
        (xto, yto): (usize, usize),
        filter: impl Fn(&u32) -> u32,
    ) {
        let (wmax, hmax) = (
            usize::saturating_sub(self.width, xto),
            usize::saturating_sub(self.height, yto),
        );

        let mut from_slice = self.slice(xx2, yy2);
        from_slice.clamp(wmax, hmax);
        let (width, height) = from_slice.dimensions();

        let data = from_slice.iter().map(filter).collect();
        let mut to = self.slice(xto..xto + width, yto..yto + height);

        to.copy_from_vec(data);
    }
}

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
        use std::ops::Bound::{self, *};

        let to_index = |bound: Bound<&usize>, auto: usize, max: usize| {
            let pixel = match bound {
                Included(x) => *x,
                Excluded(x) => usize::saturating_sub(*x, 1),
                Unbounded => auto,
            };
            usize::min(pixel, max)
        };

        let (width, height) = (self.img.width - 1, self.img.height - 1);

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
            .chunks(self.img.width)
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
            width: slice.img.width,
            offset: y * slice.img.width,
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
