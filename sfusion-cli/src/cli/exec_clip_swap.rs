use arboard::Clipboard;

use crate::{Error, Result};

pub fn convert_svg_to_fusion(svg_content: &str, options: &sfusion::FusionOptions) -> Result<String> {
	let trimmed = svg_content.trim();
	if trimmed.is_empty() {
		return Err(Error::custom("Clipboard text is empty."));
	}

	if !trimmed.to_ascii_lowercase().contains("<svg") {
		return Err(Error::custom("Clipboard content does not contain SVG markup."));
	}

	sfusion::svg_to_sfusion_with_options(trimmed, options)
		.map_err(|err| Error::custom(format!("Clipboard content is not valid SVG or conversion failed: {err}")))
}

pub fn exec_clip_swap(options: &sfusion::FusionOptions) -> Result<()> {
	let mut clipboard = Clipboard::new().map_err(|err| Error::custom(format!("Failed to open clipboard: {err}")))?;

	let text = clipboard
		.get_text()
		.map_err(|err| Error::custom(format!("Failed to read text from clipboard: {err}")))?;

	let fusion_content = convert_svg_to_fusion(&text, options)?;

	clipboard
		.set_text(&fusion_content)
		.map_err(|err| Error::custom(format!("Failed to write to clipboard: {err}")))?;

	println!("Successfully converted clipboard SVG to Fusion shape format.");

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_cli_exec_clip_swap_convert_svg_to_fusion_valid() -> Result<()> {
		// -- Setup & Fixtures
		let svg = r#"<svg viewBox="0 0 100 100"><rect width="50" height="50"/></svg>"#;
		let options = sfusion::FusionOptions::default().with_end_with_stransform(true);

		// -- Exec
		let fusion = convert_svg_to_fusion(svg, &options)?;

		// -- Check
		assert!(fusion.contains("Tools = ordered() {"));
		assert!(fusion.contains("sPolygon"));
		assert!(fusion.contains("sTransform"));

		Ok(())
	}

	#[test]
	fn test_cli_exec_clip_swap_convert_svg_to_fusion_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let not_svg = "just some random text";
		let options = sfusion::FusionOptions::default();

		// -- Exec
		let res = convert_svg_to_fusion(not_svg, &options);

		// -- Check
		assert!(res.is_err());

		Ok(())
	}
}

// endregion: --- Tests
