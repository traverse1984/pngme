use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Px {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Px {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rect(w: u32, h: u32, px: Px) -> Vec<Vec<Px>> {
        let mut row = Vec::with_capacity(w as usize);
        let mut rect = Vec::with_capacity(h as usize);

        row.resize(w as usize, px);
        rect.resize_with(h as usize, || row.clone());
        rect
    }

    pub fn hex(val: u32) -> Self {
        let [oflow, r, g, b] = val.to_be_bytes();
        if oflow > 0 {
            Self::rgba(oflow, r, g, b)
        } else {
            Self::rgb(r, g, b)
        }
    }

    pub fn hexa(val: u32) -> Self {
        let [r, g, b, a] = val.to_be_bytes();
        Self::rgba(r, g, b, a)
    }

    pub fn hex_str(val: &str) -> Self {
        let val = val.trim_start_matches("#").trim_start_matches("0x");
        let val = u32::from_str_radix(val, 16).unwrap_or(0);
        Self::hex(val)
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        let Self { r, g, b, a } = self;
        [*r, *g, *b, *a]
    }
}

impl fmt::Display for Px {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Px { r, g, b, a } = *self;
        write!(f, "{:x}", u32::from_be_bytes([r, g, b, a]))
    }
}
