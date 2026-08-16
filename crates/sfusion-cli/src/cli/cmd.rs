use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
	name = "sfusion",
	version,
	about = "Convert SVG files into DaVinci Resolve Fusion format"
)]
pub struct CliCmd {
	#[command(subcommand)]
	pub command: Option<CliSubCmd>,

	/// Input SVG file path (reads from stdin if not specified or '-')
	#[arg(short, long, global = true)]
	pub input: Option<String>,

	/// Output file path (writes to stdout if not specified or '-')
	#[arg(short, long, global = true)]
	pub output: Option<String>,

	/// Positional input SVG file path
	pub file: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum CliSubCmd {
	/// Convert an SVG file or stdin to DaVinci Resolve Fusion format
	Convert(ConvertArgs),
}

#[derive(Args, Debug, Default, Clone)]
pub struct ConvertArgs {
	/// Input SVG file path (reads from stdin if not specified or '-')
	#[arg(short, long)]
	pub input: Option<String>,

	/// Output file path (writes to stdout if not specified or '-')
	#[arg(short, long)]
	pub output: Option<String>,

	/// Positional input SVG file path
	pub file: Option<String>,
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_cli_cmd_parse_positional() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "input.svg"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		assert_eq!(cmd.file.as_deref(), Some("input.svg"));
		assert!(cmd.command.is_none());

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_flags() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "-i", "drawing.svg", "-o", "output.fusion"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		assert_eq!(cmd.input.as_deref(), Some("drawing.svg"));
		assert_eq!(cmd.output.as_deref(), Some("output.fusion"));

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_subcommand() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "convert", "-i", "logo.svg", "-o", "logo.fusion"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		if let Some(CliSubCmd::Convert(convert_args)) = cmd.command {
			assert_eq!(convert_args.input.as_deref(), Some("logo.svg"));
			assert_eq!(convert_args.output.as_deref(), Some("logo.fusion"));
		} else {
			return Err("Expected CliSubCmd::Convert".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
