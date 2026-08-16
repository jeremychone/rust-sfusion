type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_sfusion_svg_to_sfusion_simple_path() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 320 240" width="320" height="240">
			<path id="poly_1" d="M 10 20 L 30 40 Z"/>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.starts_with("{\n\tTools = ordered() {\n"));
	assert!(fusion_script.contains("poly_1 = sPolygon {"));
	assert!(fusion_script.contains("Value = 320"));
	assert!(fusion_script.contains("Value = 240"));
	assert!(fusion_script.contains("Closed = true,"));
	assert!(fusion_script.ends_with("\t}\n}\n"));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_shapes_and_merge() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 800 600">
			<g id="main_group">
				<rect id="box_shape" x="100" y="100" width="200" height="150"/>
				<circle id="circle_shape" cx="400" cy="300" r="50"/>
			</g>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("box_shape = sPolygon {"));
	assert!(fusion_script.contains("circle_shape = sPolygon {"));
	assert!(fusion_script.contains("main_group = sMerge {"));
	assert!(fusion_script.contains("SourceOp = \"box_shape\""));
	assert!(fusion_script.contains("SourceOp = \"circle_shape\""));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_polyline_and_polygon() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 500 500">
			<polyline points="20,20 40,25 60,40 80,120"/>
			<polygon id="triangle" points="200,10 250,190 160,210"/>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("poly_1 = sPolygon {"));
	assert!(fusion_script.contains("triangle = sPolygon {"));
	assert!(fusion_script.contains("loop = sMerge {"));
	assert!(fusion_script.contains("SourceOp = \"poly_1\""));
	assert!(fusion_script.contains("SourceOp = \"triangle\""));

	Ok(())
}
