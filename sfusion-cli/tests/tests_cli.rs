type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use std::fs;

#[test]
fn test_cli_to_shape_integration() -> Result<()> {
	// -- Setup & Fixtures
	let tmp_dir = std::env::temp_dir().join("sfusion_cli_integration_test");
	fs::create_dir_all(&tmp_dir)?;

	let svg_file = tmp_dir.join("test_polygon.svg");
	let svg_content = r#"<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><polygon points="50,5 90,90 10,90"/></svg>"#;
	fs::write(&svg_file, svg_content)?;

	// -- Exec
	let fusion_content = sfusion::svg_to_sfusion(svg_content)?;
	let output_file = tmp_dir.join("test_polygon.svg.fusion-path.txt");
	fs::write(&output_file, &fusion_content)?;

	// -- Check
	assert!(output_file.exists());
	let read_back = fs::read_to_string(&output_file)?;
	assert!(read_back.contains("Tools = ordered() {"));
	assert!(read_back.contains("sPolygon"));

	// fs::remove_dir_all(&tmp_dir)?; // Cleanup manually if needed

	Ok(())
}
