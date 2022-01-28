use clap::Parser;

#[derive(Parser)]
#[clap(name = "pngme")]
#[clap(bin_name = "pngme")]
pub enum PngME {
    Encode {
        file: String,
        chunk_type: String,
        message: Vec<String>,
    },
    Decode {
        file: String,
        chunk_type: String,
    },
    Remove {
        file: String,
        chunk_type: String,
    },
    Print {
        file: String,
    },
}

impl PngME {
    pub fn cmd() -> Self {
        Self::parse()
    }
}
