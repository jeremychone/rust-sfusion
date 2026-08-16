mod _tsupport;

use sfusion::ast::*;
use sfusion::svg::parse_svg;
use sfusion::svg_to_sfusion;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_style_svg_presentation_and_css_override() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 400 300" width="400" height="300">
			<rect id="styled_rect" x="10" y="10" width="100" height="80"
				fill="red" stroke="blue" stroke-width="2" stroke-linecap="round"
				style="fill: #00ff00; stroke-width: 6px; stroke-linejoin: bevel; opacity: 0.85"/>
		</svg>
	"##;

	// -- Exec
	let doc = parse_svg(svg_content)?;
	let fusion_script = svg_to_sfusion(svg_content)?;

	// -- Check
	assert_eq!(doc.elements.len(), 1);
	if let SvgElement::Rect(rect) = &doc.elements[0] {
		assert_eq!(rect.style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(0, 255, 0))));
		assert_eq!(rect.style.stroke, Some(SvgPaint::Color(SvgColor::new_rgb(0, 0, 255))));
		assert_eq!(rect.style.stroke_width, Some(6.0));
		assert_eq!(rect.style.stroke_linecap, Some(StrokeLinecap::Round));
		assert_eq!(rect.style.stroke_linejoin, Some(StrokeLinejoin::Bevel));
		assert_eq!(rect.style.opacity, Some(0.85));
	} else {
		return Err("Expected Rect element".into());
	}

	assert!(fusion_script.contains("styled_rect = sPolygon {"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.015, },"));

	Ok(())
}

#[test]
fn test_style_svg_nested_group_inheritance() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r#"
		<svg viewBox="0 0 500 500">
			<g id="root_grp" stroke-width="4" stroke="red" fill="yellow">
				<circle id="inherited_circle" cx="50" cy="50" r="25"/>
				<g id="inner_grp" stroke-width="8" stroke="green">
					<line id="inherited_line" x1="0" y1="0" x2="100" y2="100"/>
					<ellipse id="override_ellipse" cx="200" cy="200" rx="40" ry="20" style="stroke-width: 12px; fill: blue"/>
				</g>
			</g>
		</svg>
	"#;

	// -- Exec
	let doc = parse_svg(svg_content)?;
	let fusion_script = svg_to_sfusion(svg_content)?;

	// -- Check
	assert_eq!(doc.elements.len(), 1);
	assert!(fusion_script.contains("inherited_circle = sPolygon {"));
	assert!(fusion_script.contains("inherited_line = sPolygon {"));
	assert!(fusion_script.contains("override_ellipse = sPolygon {"));
	assert!(fusion_script.contains("root_grp = sMerge {"));
	assert!(fusion_script.contains("inner_grp = sMerge {"));

	// Inherited circle: stroke_width 4 / 500 = 0.008
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.008, },"));
	// Inherited line: stroke_width 8 / 500 = 0.016
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.016, },"));
	// Overridden ellipse: stroke_width 12 / 500 = 0.024
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.024, },"));

	Ok(())
}

#[test]
fn test_style_svg_gradient_defs_and_paint_url() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 600 400">
			<defs>
				<linearGradient id="linear_accent" x1="0%" y1="0%" x2="100%" y2="0%">
					<stop offset="0%" stop-color="#ff9900" stop-opacity="1"/>
					<stop offset="100%" stop-color="#ff0066" stop-opacity="0.8"/>
				</linearGradient>
				<radialGradient id="radial_glow" cx="50%" cy="50%" r="50%">
					<stop offset="0%" stop-color="white"/>
					<stop offset="100%" stop-color="transparent"/>
				</radialGradient>
			</defs>
			<rect id="bg_card" x="0" y="0" width="600" height="400" fill="url(#linear_accent)"/>
			<circle id="center_glow" cx="300" cy="200" r="100" fill="url(#radial_glow)"/>
		</svg>
	"##;

	// -- Exec
	let doc = parse_svg(svg_content)?;
	let fusion_script = svg_to_sfusion(svg_content)?;

	// -- Check
	assert_eq!(doc.defs.gradients.len(), 2);

	let lin = doc.defs.gradients.get("linear_accent").ok_or("Missing linear_accent")?;
	match lin {
		SvgGradient::Linear(l) => {
			assert_eq!(l.x1, Some(0.0));
			assert_eq!(l.x2, Some(1.0));
			assert_eq!(l.stops.len(), 2);
			assert_eq!(l.stops[0].color, SvgColor::new_rgb(255, 153, 0));
			assert_eq!(l.stops[1].opacity, Some(0.8));
		}
		_ => return Err("Expected SvgGradient::Linear".into()),
	}

	let rad = doc.defs.gradients.get("radial_glow").ok_or("Missing radial_glow")?;
	match rad {
		SvgGradient::Radial(r) => {
			assert_eq!(r.cx, Some(0.5));
			assert_eq!(r.r, Some(0.5));
			assert_eq!(r.stops.len(), 2);
			assert_eq!(r.stops[0].color, SvgColor::new_rgb(255, 255, 255));
			assert_eq!(r.stops[1].color, SvgColor::new_rgba(0, 0, 0, 0.0));
		}
		_ => return Err("Expected SvgGradient::Radial".into()),
	}

	if let SvgElement::Rect(rect) = &doc.elements[0] {
		assert_eq!(rect.style.fill, Some(SvgPaint::Url("linear_accent".to_string())));
	} else {
		return Err("Expected Rect element".into());
	}

	if let SvgElement::Circle(circle) = &doc.elements[1] {
		assert_eq!(circle.style.fill, Some(SvgPaint::Url("radial_glow".to_string())));
	} else {
		return Err("Expected Circle element".into());
	}

	assert!(fusion_script.contains("bg_card = sPolygon {"));
	assert!(fusion_script.contains("center_glow = sPolygon {"));
	assert!(fusion_script.contains("loop = sMerge {"));

	Ok(())
}

#[test]
fn test_style_svg_all_elements_with_styles() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = r##"
		<svg viewBox="0 0 1000 1000">
			<path id="path_elem" d="M 10 10 L 90 90" stroke="#123456" stroke-width="5"/>
			<rect id="rect_elem" x="100" y="100" width="80" height="80" style="stroke-width: 10px"/>
			<circle id="circle_elem" cx="300" cy="300" r="50" stroke-width="2"/>
			<ellipse id="ellipse_elem" cx="500" cy="500" rx="60" ry="30" stroke-width="4"/>
			<line id="line_elem" x1="600" y1="600" x2="700" y2="700" stroke-width="6"/>
			<polyline id="polyline_elem" points="750,750 800,850 850,750" stroke-width="8"/>
			<polygon id="polygon_elem" points="900,900 950,990 850,990" stroke-width="12"/>
		</svg>
	"##;

	// -- Exec
	let fusion_script = svg_to_sfusion(svg_content)?;

	// -- Check
	assert!(fusion_script.contains("path_elem = sPolygon {"));
	assert!(fusion_script.contains("rect_elem = sPolygon {"));
	assert!(fusion_script.contains("circle_elem = sPolygon {"));
	assert!(fusion_script.contains("ellipse_elem = sPolygon {"));
	assert!(fusion_script.contains("line_elem = sPolygon {"));
	assert!(fusion_script.contains("polyline_elem = sPolygon {"));
	assert!(fusion_script.contains("polygon_elem = sPolygon {"));

	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.005, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.01, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.002, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.004, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.006, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.008, },"));
	assert!(fusion_script.contains("BorderWidth = Input { Value = 0.012, },"));

	Ok(())
}
