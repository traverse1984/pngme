mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;
mod px;

use args::PngME::{self, *};
use png::PngError;

fn exec(command: PngME) -> Result<(), PngError> {
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

fn main() -> Result<(), PngError> {
    exec(PngME::cmd())
}
