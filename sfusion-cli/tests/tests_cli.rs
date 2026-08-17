type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use std::fs;
use std::path::Path;

#[test]
fn test_cli_to_shape_integration() -> Result<()> {
	// -- Setup & Fixtures
	let tmp_dir = Path::new("tests/data/.tmp/test_cli_to_shape_integration");
	fs::create_dir_all(tmp_dir)?;

	let svg_file = tmp_dir.join("test_polygon.svg");
	let svg_content =
		r#"<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><polygon points="50,5 90,90 10,90"/></svg>"#;
	fs::write(&svg_file, svg_content)?;

	// -- Exec
	let fusion_content = sfusion::svg_to_sfusion(svg_content)?;
	let output_file = tmp_dir.join("test_polygon.svg.fusion-shape.txt");
	fs::write(&output_file, &fusion_content)?;

	// -- Check
	assert!(output_file.exists());
	let read_back = fs::read_to_string(&output_file)?;
	assert!(read_back.contains("Tools = ordered() {"));
	assert!(read_back.contains("sPolygon"));

	// fs::remove_dir_all(&tmp_dir)?; // Cleanup manually if needed

	Ok(())
}

#[test]
fn test_cli_to_shape_with_sxf_integration() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"<svg viewBox="0 0 100 100"><rect width="80" height="80"/></svg>"#;
	let options = sfusion::FusionOptions::default().with_end_with_stransform(true);

	// -- Exec
	let fusion_content = sfusion::svg_to_sfusion_with_options(svg_content, &options)?;

	// -- Check
	assert!(fusion_content.contains("Tools = ordered() {"));
	assert!(fusion_content.contains("sPolygon"));
	assert!(fusion_content.contains("sTransform"));
	assert!(fusion_content.contains("CtrlWZoom = false"));
	assert!(fusion_content.contains("Source = \"Output\""));

	Ok(())
}

#[test]
fn test_cli_stdin_integration() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"<svg viewBox="0 0 200 200"><rect x="20" y="20" width="160" height="160"/></svg>"#;

	// -- Exec
	let fusion_content = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_content.contains("Tools = ordered() {"));
	assert!(fusion_content.contains("sPolygon"));
	assert!(fusion_content.contains("Points = {"));

	Ok(())
}

#[test]
fn test_cli_malformed_svg_err() -> Result<()> {
	// -- Setup & Fixtures
	let malformed_svg = "<svg><invalid";

	// -- Exec
	let result = sfusion::svg_to_sfusion(malformed_svg);

	// -- Check
	assert!(result.is_err());

	Ok(())
}
