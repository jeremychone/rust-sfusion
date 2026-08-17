type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_text_standalone_simple() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 400 200" width="400" height="200">
			<text id="hello_title" x="20" y="50" font-family="Roboto" font-size="28" font-weight="bold" fill="#ff0000">
				Hello World
			</text>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("hello_title = sText {"));
	assert!(fusion_script.contains("StyledText = Input { Value = \"Hello World\", },"));
	assert!(fusion_script.contains("Font = Input { Value = \"Roboto\", },"));
	assert!(fusion_script.contains("Style = Input { Value = \"Bold\", },"));
	assert!(fusion_script.contains("Red1 = Input { Value = 1, },"));
	assert!(fusion_script.contains("Green1 = Input { Value = 0, },"));
	assert!(fusion_script.contains("Blue1 = Input { Value = 0, },"));
	assert!(fusion_script.contains("VerticalJustificationNew = Input { Value = 3, },"));

	Ok(())
}

#[test]
fn test_text_with_inline_style_and_anchors() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 600 300">
			<text id="styled_label" x="300" y="150"
				style="font-family: 'Open Sans'; font-style: italic; text-anchor: middle; fill: #00ff00; opacity: 0.5">
				Centered Text
			</text>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("styled_label = sText {"));
	assert!(fusion_script.contains("StyledText = Input { Value = \"Centered Text\", },"));
	assert!(fusion_script.contains("Font = Input { Value = \"Open Sans\", },"));
	assert!(fusion_script.contains("Style = Input { Value = \"Italic\", },"));
	assert!(fusion_script.contains("Red1 = Input { Value = 0, },"));
	assert!(fusion_script.contains("Green1 = Input { Value = 1, },"));
	assert!(fusion_script.contains("Blue1 = Input { Value = 0, },"));
	assert!(fusion_script.contains("Opacity1 = Input { Value = 0.5, },"));
	assert!(fusion_script.contains("HorizontalJustificationNew = Input { Value = 3, },"));
	assert!(fusion_script.contains("HorizontalLeftCenterRight = Input { Value = 1, },"));

	Ok(())
}

#[test]
fn test_text_multiline_tspans() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 500 400">
			<text id="multiline_card" x="50" y="50" font-family="Helvetica">
				Header Title
				<tspan x="50" y="80">Subheading info</tspan>
				<tspan x="50" y="110">Footer note</tspan>
			</text>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("multiline_card = sText {"));
	assert!(fusion_script.contains("StyledText = Input { Value = \"Header TitleSubheading infoFooter note\", },"));
	assert!(fusion_script.contains("Font = Input { Value = \"Helvetica\", },"));

	Ok(())
}

#[test]
fn test_text_mixed_with_shapes_in_group() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 800 600">
			<g id="badge_group">
				<rect id="bg_rect" x="100" y="100" width="200" height="80" fill="#333333"/>
				<text id="badge_txt" x="200" y="150" font-family="Arial" font-weight="bold" fill="#ffffff">
					Action
				</text>
			</g>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("bg_rect = sPolygon {"));
	assert!(fusion_script.contains("badge_txt = sText {"));
	assert!(fusion_script.contains("badge_group = sMerge {"));
	assert!(fusion_script.contains("SourceOp = \"bg_rect\""));
	assert!(fusion_script.contains("SourceOp = \"badge_txt\""));

	Ok(())
}

#[test]
fn test_text_escaping_special_characters() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 400 200">
			<text id="quote_text" font-family="Arial">
				Quote: &quot;Hello&quot; \ Path \ Test
			</text>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("quote_text = sText {"));
	assert!(fusion_script.contains(r#"StyledText = Input { Value = "Quote: \"Hello\" \\ Path \\ Test", },"#));

	Ok(())
}

#[test]
fn test_text_nested_tspan_inline_flattening() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 200">
			<text id="brand_text" x="50" y="100" font-family="Lato-Bold, Lato, sans-serif" font-weight="700" font-size="32" fill="#ffffff">
				R<tspan fill="#ff0000">U</tspan>ST
			</text>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("brand_text = sText {"));
	assert!(fusion_script.contains("StyledText = Input { Value = \"RUST\", },"));
	assert!(fusion_script.contains("Font = Input { Value = \"Lato\", },"));
	assert!(fusion_script.contains("Style = Input { Value = \"Bold\", },"));

	Ok(())
}
