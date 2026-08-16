use std::fs;
use std::io::{self, Read, Write};

use crate::cli::cmd::ConvertArgs;
use crate::Result;

pub fn exec_convert(args: ConvertArgs) -> Result<()> {
	let input_source = args.input.or(args.file);

	let svg_content = match input_source {
		Some(ref path) if path != "-" => fs::read_to_string(path)?,
		_ => {
			let mut buffer = String::new();
			io::stdin().read_to_string(&mut buffer)?;
			buffer
		}
	};

	let fusion_content = sfusion::svg_to_sfusion(&svg_content)?;

	match args.output {
		Some(ref path) if path != "-" => {
			fs::write(path, fusion_content)?;
		}
		_ => {
			let mut stdout = io::stdout().lock();
			stdout.write_all(fusion_content.as_bytes())?;
			if !fusion_content.ends_with('\n') {
				stdout.write_all(b"\n")?;
			}
			stdout.flush()?;
		}
	}

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_cli_exec_convert_file_to_file() -> Result<()> {
		// -- Setup & Fixtures
		let tmp_dir = std::env::temp_dir().join("sfusion_cli_test_exec_convert");
		fs::create_dir_all(&tmp_dir)?;

		let input_path = tmp_dir.join("test_input.svg");
		let output_path = tmp_dir.join("test_output.txt");

		let svg_data = r#"<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40"/></svg>"#;
		fs::write(&input_path, svg_data)?;

		let args = ConvertArgs {
			input: Some(input_path.to_string_lossy().to_string()),
			output: Some(output_path.to_string_lossy().to_string()),
			file: None,
		};

		// -- Exec
		exec_convert(args)?;

		// -- Check
		let output_content = fs::read_to_string(&output_path)?;
		assert!(output_content.contains("Tools = ordered() {"));
		assert!(output_content.contains("sPolygon {"));

		// fs::remove_dir_all(&tmp_dir)?; // Cleanup manually if needed

		Ok(())
	}
}

// endregion: --- Tests
