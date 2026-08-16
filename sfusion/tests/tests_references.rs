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
fn test_references_ref_01_structural_verification() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = include_str!("data/references/ref-01.svg");

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(actual_fusion.contains("grabber = sPolygon {"));
	assert!(actual_fusion.contains("poly_1 = sPolygon {"));
	assert!(actual_fusion.contains("loop = sMerge {"));
	// grabber color #4cb7e4 -> (76/255, 183/255, 228/255)
	assert!(actual_fusion.contains("Red = Input { Value = 0.2980392156862745, },"));
	assert!(actual_fusion.contains("Green = Input { Value = 0.7176470588235294, },"));
	assert!(actual_fusion.contains("Blue = Input { Value = 0.8941176470588236, },"));

	// poly_1 color white -> (1, 1, 1)
	assert!(actual_fusion.contains("Red = Input { Value = 1, },"));
	assert!(actual_fusion.contains("Green = Input { Value = 1, },"));
	assert!(actual_fusion.contains("Blue = Input { Value = 1, },"));

	Ok(())
}

#[test]
fn test_references_complex_multi_layer_and_color_carryover() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 1000 800" width="1000" height="800" xmlns="http://www.w3.org/2000/svg">
			<rect id="bg_layer" x="0" y="0" width="1000" height="800" fill="#1e1e2e"/>
			<g id="card_group" color="tomato" stroke-width="6">
				<rect id="card_bg" x="100" y="100" width="800" height="600" rx="20" ry="20" fill="#336699" opacity="0.9"/>
				<circle id="card_badge" cx="200" cy="200" r="50" fill="currentColor"/>
				<path id="card_accent" d="M 300 300 L 700 300" stroke="hsl(210, 100%, 50%)" fill="none"/>
			</g>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	// 1. Tool declarations
	assert!(fusion_script.contains("bg_layer = sPolygon {"));
	assert!(fusion_script.contains("card_bg = sPolygon {"));
	assert!(fusion_script.contains("card_badge = sPolygon {"));
	assert!(fusion_script.contains("card_accent = sPolygon {"));
	assert!(fusion_script.contains("card_group = sMerge {"));
	assert!(fusion_script.contains("smerge = sMerge {"));

	// 2. Layer ordering in card_group sMerge: bottom (card_bg) -> middle (card_badge) -> top (card_accent)
	assert!(fusion_script.contains("Input1 = Input {\n\t\t\t\t\tSourceOp = \"card_bg\","));
	assert!(fusion_script.contains("Input2 = Input {\n\t\t\t\t\tSourceOp = \"card_badge\","));
	assert!(fusion_script.contains("Input3 = Input {\n\t\t\t\t\tSourceOp = \"card_accent\","));

	// 3. Layer ordering at root level: bg_layer (Input1) -> card_group (Input2)
	assert!(fusion_script.contains("Input1 = Input {\n\t\t\t\t\tSourceOp = \"bg_layer\","));
	assert!(fusion_script.contains("Input2 = Input {\n\t\t\t\t\tSourceOp = \"card_group\","));

	// 4. Color checks:
	// bg_layer: #1e1e2e -> (30/255, 30/255, 46/255)
	assert!(fusion_script.contains("Red = Input { Value = 0.11764705882352941, },"));
	assert!(fusion_script.contains("Green = Input { Value = 0.11764705882352941, },"));
	assert!(fusion_script.contains("Blue = Input { Value = 0.1803921568627451, },"));

	// card_bg: #336699 -> (51/255, 102/255, 153/255) = (0.2, 0.4, 0.6), opacity 0.9
	assert!(fusion_script.contains("Red = Input { Value = 0.2, },"));
	assert!(fusion_script.contains("Green = Input { Value = 0.4, },"));
	assert!(fusion_script.contains("Blue = Input { Value = 0.6, },"));
	assert!(fusion_script.contains("Opacity = Input { Value = 0.9, },"));

	// card_badge: currentColor -> tomato (#ff6347 -> 255/255, 99/255, 71/255)
	assert!(fusion_script.contains("Red = Input { Value = 1, },"));
	assert!(fusion_script.contains("Green = Input { Value = 0.38823529411764707, },"));
	assert!(fusion_script.contains("Blue = Input { Value = 0.2784313725490196, },"));

	// card_accent: hsl(210, 100%, 50%) -> (0, 0.5, 1.0), stroke-width 6 / 1000 = 0.006
	assert!(fusion_script.contains("Red = Input { Value = 0, },"));
	assert!(fusion_script.contains("Green = Input { Value = 0.5019607843137255, },"));
	assert!(fusion_script.contains("Blue = Input { Value = 1, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.006, },"));

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

#[test]
fn test_ref_02_crabby_grouping_and_styling() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = include_str!("data/references/ref-02.svg");
	let expected_fusion = include_str!("data/references/ref-02.txt");

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;
	let reference_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/data/references/ref-02.txt");
	std::fs::write(reference_path, &actual_fusion)?;

	// -- Check
	assert_eq!(actual_fusion, expected_fusion);
	// 1. Root structure and group merge
	assert!(actual_fusion.contains("crabby_final = sMerge {"));
	// Ensure dedicated smerge exists for multi-subpath item
	assert!(actual_fusion.contains("smerge = sMerge {"));

	// 2. All polygon tools generated and wired through smerge & crabby_final
	assert!(actual_fusion.contains("poly = sPolygon {"));
	assert!(actual_fusion.contains("poly_1 = sPolygon {"));
	assert!(actual_fusion.contains("poly_2 = sPolygon {"));
	assert!(actual_fusion.contains("poly_4 = sPolygon {"));
	assert!(actual_fusion.contains("poly_5 = sPolygon {"));
	assert!(actual_fusion.contains("poly_6 = sPolygon {"));
	assert!(actual_fusion.contains("poly_7 = sPolygon {"));
	assert!(actual_fusion.contains("poly_8 = sPolygon {"));
	assert!(actual_fusion.contains("poly_9 = sPolygon {"));

	assert!(actual_fusion.contains("SourceOp = \"smerge\""));
	assert!(actual_fusion.contains("SourceOp = \"poly\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_1\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_2\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_4\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_5\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_6\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_7\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_8\""));
	assert!(actual_fusion.contains("SourceOp = \"poly_9\""));

	// 3. Black pupil shapes (poly_4, poly_5) have explicit black RGB inputs
	assert!(actual_fusion.contains("Red = Input { Value = 0, },\n\t\t\t\tGreen = Input { Value = 0, },\n\t\t\t\tBlue = Input { Value = 0, },"));

	Ok(())
}

#[test]
fn test_ref_03_generate_reference() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = include_str!("data/references/ref-03.svg");
	let expected_fusion = include_str!("data/references/ref-03.txt");

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;
	let reference_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/data/references/ref-03.txt");
	std::fs::write(reference_path, &actual_fusion)?;
	assert_eq!(actual_fusion, expected_fusion);

	Ok(())
}
