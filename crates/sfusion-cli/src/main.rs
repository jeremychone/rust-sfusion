// region:    --- Modules

mod cli;
mod error;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() -> Result<()> {
	cli::execute()?;

	Ok(())
}
