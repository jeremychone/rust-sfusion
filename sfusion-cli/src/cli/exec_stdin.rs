use std::io::{self, Read, Write};

use crate::Result;

pub fn exec_stdin() -> Result<()> {
	let mut buffer = String::new();
	io::stdin().read_to_string(&mut buffer)?;

	let fusion_content = sfusion::svg_to_sfusion(&buffer)?;

	let mut stdout = io::stdout().lock();
	stdout.write_all(fusion_content.as_bytes())?;
	if !fusion_content.ends_with('\n') {
		stdout.write_all(b"\n")?;
	}
	stdout.flush()?;

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_stdin_svg_to_fusion() -> Result<()> {
		// -- Setup & Fixtures
		let svg_data = r#"<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40"/></svg>"#;

		// -- Exec
		let fusion_content = sfusion::svg_to_sfusion(svg_data)?;

		// -- Check
		assert!(fusion_content.contains("Tools = ordered() {"));
		assert!(fusion_content.contains("sPolygon {"));

		Ok(())
	}
}

// endregion: --- Tests
