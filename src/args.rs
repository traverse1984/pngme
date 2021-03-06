use clap::Parser;

#[derive(Parser)]
#[clap(name = "pngme")]
#[clap(bin_name = "pngme")]
pub enum PngME {
    Encode {
        file: String,
        chunk_type: String,
        message: Vec<String>,
        #[clap(short, long)]
        unchecked: bool,
    },
    Decode {
        file: String,
        chunk_type: String,
    },
    Remove {
        file: String,
        chunk_type: String,
        #[clap(short, long)]
        unchecked: bool,
    },
    Print {
        file: String,
    },
    Scrub {
        file: String,
    },
    Generate,
}

impl PngME {
    pub fn cmd() -> Self {
        Self::parse()
    }
}
