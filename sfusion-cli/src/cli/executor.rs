use clap::Parser as _;

use crate::cli::cmd::{CliCmd, CliSubCmd};
use crate::cli::{exec_clip_swap, exec_stdin, exec_to_shape};
use crate::Result;

pub fn execute() -> Result<()> {
	let cli_cmd = CliCmd::parse();

	match cli_cmd.command {
		Some(CliSubCmd::ToShape(args)) => {
			exec_to_shape::exec_to_shape(args)?;
		}
		Some(CliSubCmd::ClipSwap) => {
			exec_clip_swap::exec_clip_swap()?;
		}
		None => {
			exec_stdin::exec_stdin()?;
		}
	}

	Ok(())
}
