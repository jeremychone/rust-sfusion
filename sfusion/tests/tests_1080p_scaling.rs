type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_sfusion_scaling_landscape_1080p() -> Result<()> {
	// -- Setup & Fixtures
	// 1920x1080 landscape SVG should scale width to 1080 and height to 607.5
	let svg_content = r##"
		<svg viewBox="0 0 1920 1080" width="1920" height="1080">
			<rect id="landscape_rect" x="100" y="100" width="400" height="200" fill="#ff5500"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("landscape_rect = sPolygon {"));
	assert!(fusion_script.contains("MaskWidth = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 1080"));
	assert!(fusion_script.contains("MaskHeight = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 607.5"));
	assert!(fusion_script.contains("Red = Input { Value = 1, },"));
	assert!(fusion_script.contains("Blue = Input { Value = 0, },"));

	Ok(())
}

#[test]
fn test_sfusion_scaling_custom_landscape_1080p() -> Result<()> {
	// -- Setup & Fixtures
	// 800x400 landscape SVG (2:1 aspect) should scale width to 1080 and height to 540
	let svg_content = r##"
		<svg viewBox="0 0 800 400">
			<circle id="circle_wide" cx="400" cy="200" r="100" stroke="#00ff00" stroke-width="8"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("circle_wide = sPolygon {"));
	assert!(fusion_script.contains("MaskWidth = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 1080"));
	assert!(fusion_script.contains("MaskHeight = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 540"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.01, },"));
	assert!(fusion_script.contains("Green = Input { Value = 1, },"));

	Ok(())
}

#[test]
fn test_sfusion_scaling_portrait_1080p() -> Result<()> {
	// -- Setup & Fixtures
	// 600x1200 portrait SVG (1:2 aspect) should scale height to 1080 and width to 540
	let svg_content = r##"
		<svg viewBox="0 0 600 1200" width="600" height="1200">
			<rect id="portrait_card" x="50" y="100" width="500" height="1000"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("portrait_card = sPolygon {"));
	assert!(fusion_script.contains("MaskWidth = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 540"));
	assert!(fusion_script.contains("MaskHeight = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 1080"));

	Ok(())
}

#[test]
fn test_sfusion_scaling_square_1080p() -> Result<()> {
	// -- Setup & Fixtures
	// 500x500 square SVG should scale both width and height to 1080
	let svg_content = r##"
		<svg viewBox="0 0 500 500">
			<polygon id="diamond" points="250,50 450,250 250,450 50,250"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("diamond = sPolygon {"));
	assert!(fusion_script.contains("MaskWidth = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 1080"));
	assert!(fusion_script.contains("MaskHeight = Input {\n\t\t\t\t\tValue = Number {\n\t\t\t\t\t\tValue = 1080"));
	assert!(fusion_script.contains("Closed = true,"));

	Ok(())
}

#[test]
fn test_sfusion_scaling_group_hierarchy_1080p() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 1600 900">
			<g id="ui_panel">
				<rect id="bg" x="0" y="0" width="1600" height="900" fill="#112233"/>
				<circle id="indicator" cx="800" cy="450" r="50" fill="#ffffff"/>
			</g>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	// 1600x900 landscape scales to 1080 x 607.5
	assert!(fusion_script.contains("bg = sPolygon {"));
	assert!(fusion_script.contains("indicator = sPolygon {"));
	assert!(fusion_script.contains("ui_panel = sMerge {"));
	assert!(fusion_script.contains("Value = 1080"));
	assert!(fusion_script.contains("Value = 607.5"));
	assert!(fusion_script.contains("SourceOp = \"bg\""));
	assert!(fusion_script.contains("SourceOp = \"indicator\""));

	Ok(())
}
