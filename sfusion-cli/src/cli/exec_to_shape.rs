use crate::cli::cmd::ToShapeArgs;
use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn compute_output_path(input_path: &Path) -> Result<PathBuf> {
	let file_name = input_path
		.file_name()
		.and_then(|s| s.to_str())
		.ok_or_else(|| Error::custom(format!("Invalid file path: {}", input_path.display())))?;

	let out_file_name = format!("{file_name}.fusion-shape.txt");
	let output_path = match input_path.parent() {
		Some(parent) if !parent.as_os_str().is_empty() => parent.join(out_file_name),
		_ => PathBuf::from(out_file_name),
	};

	Ok(output_path)
}

pub fn exec_to_shape(args: ToShapeArgs) -> Result<()> {
	let input_path = Path::new(&args.file);
	if !input_path.exists() {
		return Err(Error::custom(format!("Input file not found: {}", input_path.display())));
	}

	let output_path = compute_output_path(input_path)?;
	let svg_content = fs::read_to_string(input_path)?;
	let fusion_content = sfusion::svg_to_sfusion(&svg_content)?;

	fs::write(&output_path, &fusion_content)?;
	println!("Output written to: {}", output_path.display());

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_cli_exec_to_shape_compute_output_path() -> Result<()> {
		// -- Setup & Fixtures
		let p1 = Path::new("drawing.svg");
		let p2 = Path::new("assets/sub/icon.svg");

		// -- Exec
		let out1 = compute_output_path(p1)?;
		let out2 = compute_output_path(p2)?;

		// -- Check
		assert_eq!(out1, PathBuf::from("drawing.svg.fusion-shape.txt"));
		assert_eq!(out2, PathBuf::from("assets/sub/icon.svg.fusion-shape.txt"));

		Ok(())
	}

	#[test]
	fn test_cli_exec_to_shape_simple() -> Result<()> {
		// -- Setup & Fixtures
		let tmp_dir = std::env::temp_dir().join("sfusion_cli_test_exec_to_shape");
		fs::create_dir_all(&tmp_dir)?;

		let input_path = tmp_dir.join("sample_icon.svg");
		let svg_data = r#"<svg viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80"/></svg>"#;
		fs::write(&input_path, svg_data)?;

		let args = ToShapeArgs {
			file: input_path.to_string_lossy().to_string(),
		};

		// -- Exec
		exec_to_shape(args)?;

		// -- Check
		let expected_output_path = tmp_dir.join("sample_icon.svg.fusion-shape.txt");
		assert!(expected_output_path.exists());

		let content = fs::read_to_string(&expected_output_path)?;
		assert!(content.contains("Tools = ordered() {"));
		assert!(content.contains("sPolygon {"));

		// fs::remove_dir_all(&tmp_dir)?; // Cleanup manually if needed

		Ok(())
	}
}

// endregion: --- Tests
