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
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliSubCmd {
	/// Convert an SVG file into DaVinci Resolve Fusion shape format (<stem>_fusion-shape.txt)
	ToShape(ToShapeArgs),

	/// Inspect system clipboard, convert SVG to DaVinci Resolve Fusion format, and write back to clipboard
	ClipSwap,
}

#[derive(Args, Debug, Clone)]
pub struct ToShapeArgs {
	/// Input SVG file path
	pub file: String,
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_cli_cmd_parse_empty() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		assert!(cmd.command.is_none());

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_to_shape() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "to-shape", "icons/star.svg"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		if let Some(CliSubCmd::ToShape(to_shape_args)) = cmd.command {
			assert_eq!(to_shape_args.file, "icons/star.svg");
		} else {
			return Err("Expected CliSubCmd::ToShape".into());
		}

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_clip_swap() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "clip-swap"];

		// -- Exec
		let cmd = CliCmd::try_parse_from(args)?;

		// -- Check
		assert!(matches!(cmd.command, Some(CliSubCmd::ClipSwap)));

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_positional_fails() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "input.svg"];

		// -- Exec
		let res = CliCmd::try_parse_from(args);

		// -- Check
		assert!(res.is_err());

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_flags_fails() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "-i", "drawing.svg", "-o", "output.fusion"];

		// -- Exec
		let res = CliCmd::try_parse_from(args);

		// -- Check
		assert!(res.is_err());

		Ok(())
	}

	#[test]
	fn test_cli_cmd_parse_convert_subcommand_fails() -> Result<()> {
		// -- Setup & Fixtures
		let args = ["sfusion", "convert", "logo.svg"];

		// -- Exec
		let res = CliCmd::try_parse_from(args);

		// -- Check
		assert!(res.is_err());

		Ok(())
	}
}

// endregion: --- Tests
