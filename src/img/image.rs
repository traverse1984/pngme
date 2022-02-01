use super::image_data::{ImageData, Quad};
use super::rect::Rect;
use super::rect_slice::RectSlice;
use crate::err::{PngErr, PngErr::*, PngRes};
use crate::png::{self, Chunk};
use std::ops::RangeBounds;

// fn checked(width: u32, height: u32, data: &Vec<u32>, filter: &Vec<u8>) -> PngRes {
//     check_dimensions(width, height)?;
//     PngErr::is_or(data.len() == png::area(width, height)?, DataLengthMismatch)?;
//     PngErr::is_or(filter.len() == height as usize, FilterLengthMismatch)
// }

pub struct Image {
    width: u32,
    height: u32,
    data: Vec<u32>,
    filter: Vec<u8>,
}

impl Quad for Image {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl ImageData for Image {
    fn to_vec(self) -> Vec<u32> {
        self.data
    }

    fn clone_to_vec(&self) -> Vec<u32> {
        self.data.clone()
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.data
            .chunks(self.width().try_into().unwrap())
            .map(|chunk| chunk.into_iter().flat_map(|px| px.to_be_bytes()))
            .zip(self.filter.iter())
            .flat_map(|(row, ft)| ft.to_be_bytes().into_iter().chain(row))
            .collect()
    }

    fn data(&self) -> &Vec<u32> {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut Vec<u32> {
        self.data.as_mut()
    }
}

impl Image {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn new(width: u32, height: u32) -> PngRes<Image> {
        Self::new_bg(width, height, 0)
    }

    pub fn new_bg(width: u32, height: u32, bg: u32) -> PngRes<Image> {
        Rect::dimensions_bounded(width, height)?;

        Ok(Self {
            width,
            height,
            data: vec![bg; png::area(width, height)?],
            filter: vec![0; height as usize],
        })
    }

    pub fn slice(
        &mut self,
        xx2: impl RangeBounds<u32>,
        yy2: impl RangeBounds<u32>,
    ) -> RectSlice<'_> {
        RectSlice::new(self, Rect::from_range(xx2, yy2))
    }

    fn from_parts(width: u32, height: u32, data: Vec<u32>, filter: Vec<u8>) -> PngRes<Image> {
        Ok(Self {
            width,
            height,
            data,
            filter,
        })
    }

    pub fn from_vec(height: u32, data: Vec<u32>) -> PngRes<Self> {
        let width = usize::checked_div(data.len(), png::to_usize(height)?).ok_or(IntOverflow)?;
        let width = png::to_u32(width)?;

        Self::from_parts(width, height, data, vec![0; height as usize])
    }

    pub fn from_vec_2d(data: Vec<Vec<u32>>) -> PngRes<Self> {
        let height = data.len();
        let width = data.get(0).ok_or_else(|| ZeroWidth)?.len();

        if data.iter().any(|row| row.len() != width) {
            return Err(WidthMismatch);
        }

        Self::from_parts(
            png::to_u32(width)?,
            png::to_u32(height)?,
            data.into_iter().flatten().collect(),
            vec![0; height],
        )
    }

    // pub fn copy(
    //     &mut self,
    //     (xx2, yy2): (impl RangeBounds<usize>, impl RangeBounds<usize>),
    //     (xto, yto): (u32, u32),
    // ) {
    //     self.copy_filter((xx2, yy2), (xto, yto), |x| *x);
    //     //let slice = self.slice(xx2, yy2).fill(0);
    // }

    // pub fn copy_filter(
    //     &mut self,
    //     (xx2, yy2): (impl RangeBounds<usize>, impl RangeBounds<usize>),
    //     (xto, yto): (u32, u32),
    //     filter: impl Fn(&u32) -> u32,
    // ) {
    //     let (wmax, hmax) = (
    //         u32::saturating_sub(self.width(), xto),
    //         u32::saturating_sub(self.height(), yto),
    //     );

    //     let mut from_slice = self.slice(xx2, yy2);
    //     from_slice.clamp(wmax, hmax);
    //     let (width, height) = from_slice.dimensions();

    //     let data = from_slice.iter().map(filter).collect();
    //     let mut to = self.slice(xto..xto + width, yto..yto + height);

    //     to.copy_from_vec(data);
    // }
}
