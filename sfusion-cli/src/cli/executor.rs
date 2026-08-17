use clap::Parser as _;

use crate::cli::cmd::{CliCmd, CliSubCmd};
use crate::cli::{exec_clip_swap, exec_stdin, exec_to_shape};
use crate::Result;

pub fn execute() -> Result<()> {
	let cli_cmd = CliCmd::parse();
	let options = sfusion::FusionOptions::default().with_end_with_stransform(cli_cmd.sxf);

	match cli_cmd.command {
		Some(CliSubCmd::ToShape(args)) => {
			exec_to_shape::exec_to_shape(args, &options)?;
		}
		Some(CliSubCmd::ClipSwap) => {
			exec_clip_swap::exec_clip_swap(&options)?;
		}
		None => {
			exec_stdin::exec_stdin(&options)?;
		}
	}

	Ok(())
}
