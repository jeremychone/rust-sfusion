type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_ref_01() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = include_str!("data/references/ref-01.svg");
	let _expected_fusion = include_str!("data/references/ref-01.txt");

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(actual_fusion.contains("Tools = ordered() {"));
	assert!(actual_fusion.contains("sPolygon"));
	assert!(actual_fusion.contains("sMerge"));

	Ok(())
}

#[test]
fn test_references_colors_and_opacity_integration() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"<svg width="200" height="200" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
		<rect id="box_red" x="10" y="10" width="80" height="80" fill="#ff0000" opacity="0.5"/>
		<circle id="circle_gold" cx="150" cy="150" r="40" style="fill: gold;"/>
		<path id="line_blue" d="M 10 190 L 190 10" stroke="rgb(0, 128, 255)" stroke-width="4" stroke-opacity="0.8" fill="none"/>
	</svg>"##;

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(actual_fusion.contains("box_red = sPolygon {"));
	assert!(actual_fusion.contains("Red = Input { Value = 1, },"));
	assert!(actual_fusion.contains("Green = Input { Value = 0, },"));
	assert!(actual_fusion.contains("Blue = Input { Value = 0, },"));
	assert!(actual_fusion.contains("Opacity = Input { Value = 0.5, },"));

	assert!(actual_fusion.contains("circle_gold = sPolygon {"));
	// gold is #ffd700 (255, 215, 0)
	assert!(actual_fusion.contains("Red = Input { Value = 1, },"));
	assert!(actual_fusion.contains("Green = Input { Value = 0.8431372549019608, },"));
	assert!(actual_fusion.contains("Blue = Input { Value = 0, },"));

	assert!(actual_fusion.contains("line_blue = sPolygon {"));
	assert!(actual_fusion.contains("Red = Input { Value = 0, },"));
	assert!(actual_fusion.contains("Green = Input { Value = 0.5019607843137255, },"));
	assert!(actual_fusion.contains("Blue = Input { Value = 1, },"));
	assert!(actual_fusion.contains("Opacity = Input { Value = 0.8, },"));
	assert!(actual_fusion.contains("BorderWidth = Input { Value = 0.02, },"));

	Ok(())
}
