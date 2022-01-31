use super::RectSlice;
use crate::err::{PngErr, PngErr::*, PngRes};
use crate::png::Chunk;
use std::ops::RangeBounds;

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

    pub fn new(width: usize, height: usize) -> PngRes<ImageData> {
        Self::new_bg(width, height, 0)
    }

    pub fn new_bg(width: usize, height: usize, bg: u32) -> PngRes<ImageData> {
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

    fn checked_dimensions(width: usize, height: usize) -> PngRes {
        PngErr::not_or(width == 0, ZeroWidth)?;
        PngErr::not_or(height == 0, ZeroHeight)?;
        PngErr::not_or(width > Chunk::INT_MAX, WidthOverflow)?;
        PngErr::not_or(height > Chunk::INT_MAX, HeightOverflow)
    }

    fn checked(width: usize, height: usize, data: &Vec<u32>, filter: &Vec<u8>) -> PngRes {
        Self::checked_dimensions(width, height)?;
        PngErr::is_or(data.len() == width * height, DataLengthMismatch)?;
        PngErr::is_or(filter.len() == height, FilterLengthMismatch)
    }

    fn from_parts(
        width: usize,
        height: usize,
        data: Vec<u32>,
        filter: Vec<u8>,
    ) -> PngRes<ImageData> {
        Self::checked(width, height, &data, &filter)?;

        Ok(Self {
            width,
            height,
            data,
            filter,
        })
    }

    pub fn from_vec_2d(data: Vec<Vec<u32>>) -> PngRes<ImageData> {
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
