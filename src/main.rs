mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use args::PngME::{self, *};
use png::PngError;

fn exec(command: PngME) -> Result<(), PngError> {
    Ok(match command {
        Encode {
            file,
            chunk_type,
            message,
        } => commands::encode(&file, &chunk_type, &message.join(" "))?,
        Decode { file, chunk_type } => {
            let message = commands::decode(&file, &chunk_type)?;
            println!("{}", message);
        }
        Remove { file, chunk_type } => {
            commands::remove(&file, &chunk_type)?;
        }
        Print { file } => {
            let content = commands::print(&file)?;
            println!("{}", content);
        }
    })
}

fn main() -> Result<(), PngError> {
    exec(PngME::cmd())
}
