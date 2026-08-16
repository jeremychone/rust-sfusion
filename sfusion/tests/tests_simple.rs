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
	assert!(fusion_script.contains("Value = 1080"));
	assert!(fusion_script.contains("Value = 810"));
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
	assert!(fusion_script.contains("smerge = sMerge {"));
	assert!(fusion_script.contains("SourceOp = \"poly_1\""));
	assert!(fusion_script.contains("SourceOp = \"triangle\""));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_stroke_width() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 100 100">
			<g stroke-width="2">
				<path id="path_inherited" d="M 0 0 L 10 10"/>
				<path id="path_styled" d="M 10 10 L 20 20" style="stroke-width: 5px"/>
			</g>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("path_inherited = sPolygon {"));
	assert!(fusion_script.contains("path_styled = sPolygon {"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.02, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.05, },"));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_nested_group_transforms() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 100 100" width="100" height="100">
			<g id="grp_1" transform="matrix(1 0 0 1 10 20)">
				<g id="grp_2" transform="matrix(2 0 0 2 0 0)">
					<line id="seg" x1="0" y1="0" x2="10" y2="10"/>
				</g>
			</g>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("grp_1 = sPolygon {"));
	// Start point: x = 10, y = 20
	// End point: x = 10 + 20 = 30, y = 20 + 20 = 40
	assert!(fusion_script.contains("Points = {"));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_sanitized_keys() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 400 400">
			<g id="3d-main layer">
				<path id="sub-item.1" d="M 10 10 L 20 20 Z"/>
				<rect id="icon#2@home!" x="30" y="30" width="10" height="10"/>
			</g>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("_3d_main_layer = sMerge {"));
	assert!(fusion_script.contains("sub_item_1 = sPolygon {"));
	assert!(fusion_script.contains("icon_2_home_ = sPolygon {"));
	assert!(fusion_script.contains("SourceOp = \"sub_item_1\""));
	assert!(fusion_script.contains("SourceOp = \"icon_2_home_\""));

	Ok(())
}

#[test]
fn test_sfusion_svg_to_sfusion_layer_ordering_z_index() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 500 500">
			<rect id="bg_layer" x="0" y="0" width="500" height="500"/>
			<circle id="mid_layer" cx="250" cy="250" r="100"/>
			<path id="top_layer" d="M 200 200 L 300 300"/>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("smerge = sMerge {"));
	assert!(fusion_script.contains("Input1 = Input {\n\t\t\t\t\tSourceOp = \"bg_layer\","));
	assert!(fusion_script.contains("Input2 = Input {\n\t\t\t\t\tSourceOp = \"mid_layer\","));
	assert!(fusion_script.contains("Input3 = Input {\n\t\t\t\t\tSourceOp = \"top_layer\","));

	Ok(())
}
