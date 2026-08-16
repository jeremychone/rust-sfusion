use clap::Parser as _;

use crate::cli::cmd::{CliCmd, CliSubCmd, ConvertArgs};
use crate::cli::exec_convert;
use crate::Result;

pub fn execute() -> Result<()> {
	let cli_cmd = CliCmd::parse();

	match cli_cmd.command {
		Some(CliSubCmd::Convert(mut args)) => {
			if args.input.is_none() {
				args.input = cli_cmd.input;
			}
			if args.output.is_none() {
				args.output = cli_cmd.output;
			}
			if args.file.is_none() {
				args.file = cli_cmd.file;
			}
			exec_convert::exec_convert(args)?;
		}
		None => {
			let args = ConvertArgs {
				input: cli_cmd.input,
				output: cli_cmd.output,
				file: cli_cmd.file,
			};
			exec_convert::exec_convert(args)?;
		}
	}

	Ok(())
}
