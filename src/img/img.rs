use super::{rect::Rect, rect_slice::RectSlice};
use crate::{
    area, calc, convert,
    err::{PngErr::*, *},
    fs,
    png::{self, Chunk, Png},
    Image, Quad,
};

use std::ops::RangeBounds;

#[derive(Debug, Clone)]
pub struct Img {
    width: u32,
    height: u32,
    data: Vec<u32>,
    filter: Vec<u8>,
}

impl Quad for Img {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl Image for Img {
    fn to_vec(self) -> Vec<u32> {
        self.data
    }

    fn clone_to_vec(&self) -> Vec<u32> {
        self.data.clone()
    }
}

impl Img {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn new(width: u32, height: u32) -> Img {
        Self::new_bg(width, height, 0)
    }

    pub fn new_bg(width: u32, height: u32, bg: u32) -> Img {
        Self {
            width,
            height,
            data: vec![bg; area!(width, height)],
            filter: vec![0; convert!(ex usize; height)],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.data
            .chunks(self.width().try_into().unwrap())
            .map(|chunk| chunk.into_iter().flat_map(|px| px.to_be_bytes()))
            .zip(self.filter.iter())
            .flat_map(|(row, ft)| ft.to_be_bytes().into_iter().chain(row))
            .collect()
    }

    pub fn data(&self) -> &Vec<u32> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u32> {
        self.data.as_mut()
    }

    pub fn slice(
        &mut self,
        xx2: impl RangeBounds<u32>,
        yy2: impl RangeBounds<u32>,
    ) -> RectSlice<'_> {
        let rs = RectSlice::new(
            self,
            Rect::from_range(xx2, yy2).constrain(self.width(), self.height()),
        );
        rs
    }

    fn from_parts(width: u32, height: u32, data: Vec<u32>, filter: Vec<u8>) -> PngRes<Img> {
        Ok(Self {
            width,
            height,
            data,
            filter,
        })
    }

    pub fn from_vec(width: u32, height: u32, data: Vec<u32>) -> Self {
        let area = area!(width, height);
        let mut data = data;
        if area > data.len() {
            data.append(&mut vec![0u32; area - data.len()]);
        }

        Self {
            width,
            height,
            data,
            filter: vec![0; convert!(ex usize; height)],
        }
    }

    pub fn from_vec_2d(data: Vec<Vec<u32>>) -> PngRes<Self> {
        let height = data.len();
        let width = data.get(0).ok_or_else(|| ZeroWidth)?.len();

        if data.iter().any(|row| row.len() != width) {
            return Err(WidthMismatch);
        }

        Self::from_parts(
            convert!(u32; width)?,
            convert!(u32; height)?,
            data.into_iter().flatten().collect(),
            vec![0; height],
        )
    }

    pub fn reset_filter(&mut self) {
        self.filter = vec![0; convert!(ex usize; self.height)];
    }
}

impl TryFrom<Png> for Img {
    type Error = PngErr;
    fn try_from(png: Png) -> PngRes<Img> {
        let header = png.chunk_by_type("IHDR").ok_or(PngErr::InvalidHeader)?;
        let (width, height) = Chunk::ihdr_to_dimensions(&header)?;

        // std::mem::transmute...

        let chunk_size = convert!(usize; width)?
            .checked_mul(4)
            .ok_or(PngErr::IntOverflow)?
            .checked_add(1)
            .ok_or(PngErr::IntOverflow)?;

        let data = fs::decompress(
            png.chunks()
                .into_iter()
                .filter(|&chunk| chunk.chunk_type().to_string() == "IDAT")
                .map(|chunk| chunk.data())
                .flatten()
                .copied()
                .collect::<Vec<u8>>()
                .as_slice(),
        )?;

        let scans = data
            .len()
            .checked_div(chunk_size)
            .ok_or(PngErr::DataLengthMismatch)?;

        let uheight = convert!(usize; height)?;
        if scans != uheight {
            return Err(PngErr::DataLengthMismatch);
        }

        let mut filter = Vec::with_capacity(uheight);
        let data: Vec<u32> = data
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, scan)| {
                filter.push(scan[0]);
                (&scan[1..])
                    .chunks(4)
                    .map(|px| u32::from_be_bytes(png::segment4(px).unwrap()))
            })
            .flatten()
            .collect();

        Self::from_parts(width, height, data, filter)
    }
}
