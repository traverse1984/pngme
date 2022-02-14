use crate::{
    col, convert,
    err::*,
    img::Img,
    png::{Chunk, Png},
    Color, Quad,
};

pub fn encode(filename: &str, chunk_type: &str, message: &str) -> PngRes {
    Png::load(filename)?
        .encode(chunk_type, message)?
        .save(filename)
}

pub fn encode_unchecked(filename: &str, chunk_type: &str, message: &str) -> PngRes {
    Png::load(filename)?
        .encode_unchecked(chunk_type, message)?
        .save(filename)
}

pub fn decode(filename: &str, chunk_type: &str) -> PngRes<String> {
    Png::load(filename)?.decode(chunk_type)
}

pub fn remove(filename: &str, chunk_type: &str) -> PngRes {
    Png::load(filename)?.discard(chunk_type)?.save(filename)
}

pub fn remove_unchecked(filename: &str, chunk_type: &str) -> PngRes {
    Png::load(filename)?
        .discard_unchecked(chunk_type)?
        .save(filename)
}

pub fn print(filename: &str) -> PngRes<String> {
    let png = Png::load(filename)?;
    let ihdr = match png.chunk_by_type("IHDR") {
        Some(ihdr) => match Chunk::ihdr_to_dimensions(ihdr) {
            Ok((width, height)) => format!("Image Dimensions: {}x{}", width, height),
            Err(_) => String::from("Image contains invalid IHDR chunk!"),
        },
        None => String::from("Image does not contain IHDR chunk."),
    };

    Ok(format!("{}\n{}", png.to_string(), ihdr))
}

pub fn scrub(filename: &str) -> PngRes {
    let mut png = Png::load(filename)?;
    png.scrub();
    png.save(filename)
}

pub fn generate() -> PngRes {
    let mut gradient = Img::new(600, 600);

    let mut slice = gradient.slice(0..=2, 0..=2);
    for a in 0..6 {
        for b in 0..6 {
            for c in 0..50 {
                for d in 0..50 {
                    let color = col!(((1 + a) * (1 + b)) * 7, (c + 1) * 5, (d + 1) * 5);
                    let (w, x, y, z) = convert!(ex u32; a, b,c, d);

                    slice.pos(w * 100 + y * 2, x * 100 + z * 2).fill(color);
                }
            }
        }
    }

    let mut gradient_png = Png::from_img(gradient)?;

    gradient_png
        .encode("pgMe", "I'm the gradients image.")?
        .encode_unchecked("RUST", "I'm not as critical as I appear...")?
        .save("gradients.png")?;

    let mut squares = gradient_png.to_img()?;
    let filter = |from: &u32, to: &u32| from ^ to | 0xFF;

    let small = squares
        .slice(..=250, ..=250)
        .copy_each(0, 350, filter)
        .pos(0, 350)
        .copy_each(350, 350, filter)
        .pos(350, 350)
        .clone_to_img();

    squares
        .slice(.., ..)
        .fill(col!(0xFFFFFF))
        .xywh((50, 50), small.width(), small.height())
        .copy_from(&small.data())
        .copy(50, 300)
        .copy(300, 50)
        .copy(300, 300);

    Png::from_img(squares)?
        .encode("pgMe", "I'm the squares image.")?
        .save("squares.png")?;

    Png::from_img(small)?
        .encode("pgMe", "I'm the small image.")?
        .save("small.png")
}
