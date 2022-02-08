mod args;
mod commands;
mod err;
mod img;
mod png;

mod traits;

#[macro_use]
mod macros;

use args::PngME::{self, *};

pub(crate) use err::*;
pub(crate) use traits::*;

pub const INT_MAX: u32 = 2147483648;

fn exec(command: PngME) -> PngRes {
    Ok(match command {
        Encode {
            file,
            chunk_type,
            message,
            unchecked,
        } => {
            let encode = if unchecked {
                commands::encode_unchecked
            } else {
                commands::encode
            };

            encode(&file, &chunk_type, &message.join(" "))?;
        }
        Decode { file, chunk_type } => {
            let message = commands::decode(&file, &chunk_type)?;
            println!("{}", message);
        }
        Remove {
            file,
            chunk_type,
            unchecked,
        } => {
            let remove = if unchecked {
                commands::remove_unchecked
            } else {
                commands::remove
            };
            remove(&file, &chunk_type)?;
        }
        Print { file } => {
            let content = commands::print(&file)?;
            println!("{}", content);
        }
        Scrub { file } => commands::scrub(&file)?,
        Test => commands::test()?,
    })
}

fn main() -> PngRes {
    let hi = 3;
    let lo = 4;

    Ok(())
    //exec(PngME::cmd())
}
