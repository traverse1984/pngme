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
