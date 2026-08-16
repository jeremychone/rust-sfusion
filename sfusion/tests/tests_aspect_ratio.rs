type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_aspect_ratio_circle_proportions() -> Result<()> {
	// -- Setup & Fixtures
	// A 100x100 SVG with a circle centered at (50, 50) and radius 20.
	let svg_content = r##"
		<svg viewBox="0 0 100 100" width="100" height="100">
			<circle id="test_circle" cx="50" cy="50" r="20" fill="#ff0000"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("test_circle = sPolygon {"));
	// Under 1:1 isotropic scaling with 100x100 viewBox:
	// Center is (50, 50) -> Fusion (0, 0)
	// Right point (70, 50) -> rel_x = 20, nx = 20 / 100 = 0.2
	// Top point (50, 30) -> rel_y = -20, ny = -(-20) / 100 = 0.2
	// Ratio of X radius (0.2) to Y radius (0.2) is exactly 1:1 isotropic
	assert!(fusion_script.contains("Y = 0.2"));
	assert!(fusion_script.contains("X = 0.2"));
	assert!(fusion_script.contains("Y = -0.2"));
	assert!(fusion_script.contains("X = -0.2"));

	Ok(())
}

#[test]
fn test_aspect_ratio_square_proportions() -> Result<()> {
	// -- Setup & Fixtures
	// A 200x200 SVG with a square at (50, 50) of size 100x100.
	let svg_content = r#"
		<svg viewBox="0 0 200 200">
			<rect id="square_box" x="50" y="50" width="100" height="100"/>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("square_box = sPolygon {"));
	// Center of viewBox is (100, 100)
	// Top-left corner (50, 50) -> rel_x = -50, rel_y = -50
	// nx = -50 / 200 = -0.25
	// ny = -(-50) / 200 = 0.25
	// Bottom-right corner (150, 150) -> rel_x = 50, rel_y = 50
	// nx = 50 / 200 = 0.25
	// ny = -50 / 200 = -0.25
	assert!(fusion_script.contains("X = -0.25, Y = 0.25"));
	assert!(fusion_script.contains("X = 0.25, Y = 0.25"));
	assert!(fusion_script.contains("X = 0.25, Y = -0.25"));
	assert!(fusion_script.contains("X = -0.25, Y = -0.25"));

	Ok(())
}

#[test]
fn test_aspect_ratio_nested_transforms_and_curves() -> Result<()> {
	// -- Setup & Fixtures
	// Nested groups with rotate and translate, applying to a path with cubic beziers.
	let svg_content = r#"
		<svg viewBox="0 0 400 400">
			<g id="grp_root" transform="translate(100, 100)">
				<g id="grp_child" transform="scale(0.5, 0.5)">
					<path id="curve_item" d="M 0 0 C 50 0, 50 100, 100 100"/>
				</g>
			</g>
		</svg>
	"#;

	// -- Exec
	let fusion_script = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("grp_root = sPolygon {"));
	assert!(fusion_script.contains("LX ="));
	assert!(fusion_script.contains("RX ="));
	assert!(!fusion_script.contains("sMerge"));

	Ok(())
}
