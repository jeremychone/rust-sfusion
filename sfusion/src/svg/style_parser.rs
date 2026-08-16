use std::collections::HashMap;

use crate::ast::{FillRule, StrokeLinecap, StrokeLinejoin, SvgColor, SvgPaint, SvgStyle};

// region:    --- Public Functions

/// Parses an SVG / CSS color string into `SvgColor`.
pub fn parse_color(s: &str) -> Option<SvgColor> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	if s.eq_ignore_ascii_case("transparent") {
		return Some(SvgColor::new_rgba(0, 0, 0, 0.0));
	}

	if let Some(c) = parse_named_color(s) {
		return Some(c);
	}

	if s.starts_with('#') {
		return parse_hex_color(s);
	}

	if s.starts_with("rgb(") || s.starts_with("rgba(") {
		return parse_rgb_fn(s);
	}

	None
}

/// Parses an SVG paint value (color, url, none, or currentColor).
pub fn parse_paint(s: &str) -> Option<SvgPaint> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	if s.eq_ignore_ascii_case("none") {
		return Some(SvgPaint::None);
	}

	if s.eq_ignore_ascii_case("currentcolor") {
		return Some(SvgPaint::CurrentColor);
	}

	if s.starts_with("url(") && s.ends_with(')') {
		let inner = &s[4..s.len() - 1].trim();
		let unquoted = inner.trim_matches(['\'', '"']).trim();
		let id = unquoted.strip_prefix('#').unwrap_or(unquoted);
		return Some(SvgPaint::Url(id.to_string()));
	}

	parse_color(s).map(SvgPaint::Color)
}

/// Parses a CSS inline style string (e.g., `fill: red; stroke-width: 2px`) into an `SvgStyle`.
pub fn parse_style_str(s: &str) -> SvgStyle {
	let mut style = SvgStyle::default();
	let mut extra = HashMap::new();

	for decl in s.split(';') {
		let decl = decl.trim();
		if decl.is_empty() {
			continue;
		}

		let mut parts = decl.splitn(2, ':');
		let key = match parts.next() {
			Some(k) => k.trim().to_ascii_lowercase(),
			None => continue,
		};
		let val = match parts.next() {
			Some(v) => v.trim(),
			None => continue,
		};

		match key.as_str() {
			"fill" => style.fill = parse_paint(val),
			"fill-opacity" => style.fill_opacity = parse_dimension(val),
			"fill-rule" => style.fill_rule = parse_fill_rule(val),
			"stroke" => style.stroke = parse_paint(val),
			"stroke-width" => style.stroke_width = parse_dimension(val),
			"stroke-opacity" => style.stroke_opacity = parse_dimension(val),
			"stroke-linecap" => style.stroke_linecap = parse_stroke_linecap(val),
			"stroke-linejoin" => style.stroke_linejoin = parse_stroke_linejoin(val),
			"stroke-miterlimit" => style.stroke_miterlimit = parse_dimension(val),
			"stroke-dasharray" => style.stroke_dasharray = parse_stroke_dasharray(val),
			"stroke-dashoffset" => style.stroke_dashoffset = parse_dimension(val),
			"opacity" => style.opacity = parse_dimension(val),
			_ => {
				extra.insert(key, val.to_string());
			}
		}
	}

	if !extra.is_empty() {
		style.extra = Some(extra);
	}

	style
}

/// Parses a fill rule string ("nonzero", "evenodd").
pub fn parse_fill_rule(s: &str) -> Option<FillRule> {
	match s.trim().to_ascii_lowercase().as_str() {
		"nonzero" => Some(FillRule::NonZero),
		"evenodd" => Some(FillRule::EvenOdd),
		_ => None,
	}
}

/// Parses a stroke linecap string ("butt", "round", "square").
pub fn parse_stroke_linecap(s: &str) -> Option<StrokeLinecap> {
	match s.trim().to_ascii_lowercase().as_str() {
		"butt" => Some(StrokeLinecap::Butt),
		"round" => Some(StrokeLinecap::Round),
		"square" => Some(StrokeLinecap::Square),
		_ => None,
	}
}

/// Parses a stroke linejoin string ("miter", "round", "bevel").
pub fn parse_stroke_linejoin(s: &str) -> Option<StrokeLinejoin> {
	match s.trim().to_ascii_lowercase().as_str() {
		"miter" => Some(StrokeLinejoin::Miter),
		"round" => Some(StrokeLinejoin::Round),
		"bevel" => Some(StrokeLinejoin::Bevel),
		_ => None,
	}
}

/// Parses stroke dasharray list of numbers or "none".
pub fn parse_stroke_dasharray(s: &str) -> Option<Vec<f64>> {
	let s = s.trim();
	if s.is_empty() || s.eq_ignore_ascii_case("none") {
		return None;
	}

	let nums: Vec<f64> = s
		.split([' ', ',', '\t'])
		.filter(|item| !item.is_empty())
		.filter_map(parse_dimension)
		.collect();

	if nums.is_empty() {
		None
	} else {
		Some(nums)
	}
}

/// Parses dimension numbers removing px, pt, % or unit suffixes.
pub fn parse_dimension(s: &str) -> Option<f64> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	if let Some(percent_val) = s.strip_suffix('%') {
		let num = percent_val.trim().parse::<f64>().ok()?;
		return Some(num / 100.0);
	}

	let s = s
		.trim_end_matches("px")
		.trim_end_matches("pt")
		.trim_end_matches("em")
		.trim_end_matches("rem")
		.trim();

	s.parse::<f64>().ok()
}

// endregion: --- Public Functions

// region:    --- Support

fn parse_hex_color(s: &str) -> Option<SvgColor> {
	let hex = s.strip_prefix('#')?;
	match hex.len() {
		3 => {
			let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
			let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
			let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
			Some(SvgColor::new_rgb(r, g, b))
		}
		4 => {
			let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
			let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
			let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
			let a_byte = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
			Some(SvgColor::new_rgba(r, g, b, a_byte as f64 / 255.0))
		}
		6 => {
			let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
			let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
			let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
			Some(SvgColor::new_rgb(r, g, b))
		}
		8 => {
			let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
			let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
			let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
			let a_byte = u8::from_str_radix(&hex[6..8], 16).ok()?;
			Some(SvgColor::new_rgba(r, g, b, a_byte as f64 / 255.0))
		}
		_ => None,
	}
}

fn parse_rgb_fn(s: &str) -> Option<SvgColor> {
	let open = s.find('(')?;
	let close = s.rfind(')')?;
	if close <= open {
		return None;
	}

	let args_str = &s[open + 1..close];
	let cleaned = args_str.replace('/', ",");
	let parts: Vec<&str> = cleaned
		.split([',', ' ', '\t'])
		.filter(|item| !item.is_empty())
		.collect();

	if parts.len() < 3 {
		return None;
	}

	let r = parse_color_component(parts[0])?;
	let g = parse_color_component(parts[1])?;
	let b = parse_color_component(parts[2])?;

	let a = if parts.len() >= 4 {
		parse_alpha_component(parts[3])?
	} else {
		1.0
	};

	Some(SvgColor::new_rgba(r, g, b, a))
}

fn parse_color_component(s: &str) -> Option<u8> {
	let s = s.trim();
	if let Some(pct) = s.strip_suffix('%') {
		let num = pct.trim().parse::<f64>().ok()?;
		let clamped = (num / 100.0 * 255.0).clamp(0.0, 255.0);
		Some(clamped.round() as u8)
	} else {
		let num = s.parse::<f64>().ok()?;
		Some(num.clamp(0.0, 255.0).round() as u8)
	}
}

fn parse_alpha_component(s: &str) -> Option<f64> {
	let s = s.trim();
	if let Some(pct) = s.strip_suffix('%') {
		let num = pct.trim().parse::<f64>().ok()?;
		Some((num / 100.0).clamp(0.0, 1.0))
	} else {
		let num = s.parse::<f64>().ok()?;
		Some(num.clamp(0.0, 1.0))
	}
}

fn parse_named_color(s: &str) -> Option<SvgColor> {
	let rgb = match s.to_ascii_lowercase().as_str() {
		"black" => (0, 0, 0),
		"white" => (255, 255, 255),
		"red" => (255, 0, 0),
		"lime" | "green" => (0, 255, 0),
		"blue" => (0, 0, 255),
		"yellow" => (255, 255, 0),
		"cyan" | "aqua" => (0, 255, 255),
		"magenta" | "fuchsia" => (255, 0, 255),
		"silver" => (192, 192, 192),
		"gray" | "grey" => (128, 128, 128),
		"maroon" => (128, 0, 0),
		"olive" => (128, 128, 0),
		"darkgreen" => (0, 100, 0),
		"purple" => (128, 0, 128),
		"teal" => (0, 128, 128),
		"navy" => (0, 0, 128),
		"orange" => (255, 165, 0),
		"pink" => (255, 192, 203),
		"brown" => (165, 42, 42),
		"gold" => (255, 215, 0),
		"coral" => (255, 127, 80),
		"salmon" => (250, 128, 114),
		"khaki" => (240, 230, 140),
		"plum" => (221, 160, 221),
		"violet" => (238, 130, 238),
		"indigo" => (75, 0, 130),
		"turquoise" => (64, 224, 208),
		"skyblue" => (135, 206, 235),
		"tan" => (210, 180, 140),
		"chocolate" => (210, 105, 30),
		"crimson" => (220, 20, 60),
		"tomato" => (255, 99, 71),
		"darkgray" | "darkgrey" => (169, 169, 169),
		"lightgray" | "lightgrey" => (211, 211, 211),
		_ => return None,
	};

	Some(SvgColor::new_rgb(rgb.0, rgb.1, rgb.2))
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_svg_style_parser_colors() -> Result<()> {
		// -- Setup & Fixtures
		let red_hex = parse_color("#ff0000").ok_or("Failed parsing #ff0000")?;
		let red_short = parse_color("#f00").ok_or("Failed parsing #f00")?;
		let red_name = parse_color("red").ok_or("Failed parsing red")?;
		let red_rgb = parse_color("rgb(255, 0, 0)").ok_or("Failed parsing rgb")?;
		let red_rgba = parse_color("rgba(255, 0, 0, 0.5)").ok_or("Failed parsing rgba")?;

		// -- Check
		assert_eq!(red_hex, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_short, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_name, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_rgb, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_rgba, SvgColor::new_rgba(255, 0, 0, 0.5));

		Ok(())
	}

	#[test]
	fn test_svg_style_parser_paints() -> Result<()> {
		// -- Setup & Fixtures & Check
		assert_eq!(parse_paint("none"), Some(SvgPaint::None));
		assert_eq!(parse_paint("currentColor"), Some(SvgPaint::CurrentColor));
		assert_eq!(parse_paint("url(#grad1)"), Some(SvgPaint::Url("grad1".to_string())));
		assert_eq!(parse_paint("url('#grad2')"), Some(SvgPaint::Url("grad2".to_string())));
		assert_eq!(
			parse_paint("#00ff00"),
			Some(SvgPaint::Color(SvgColor::new_rgb(0, 255, 0)))
		);

		Ok(())
	}

	#[test]
	fn test_svg_style_parser_style_str() -> Result<()> {
		// -- Setup & Fixtures
		let style_str = "fill: #112233; stroke: url(#g1); stroke-width: 3.5px; opacity: 0.8; stroke-linecap: round; stroke-linejoin: bevel; fill-rule: evenodd; stroke-dasharray: 5, 10, 15; custom-prop: custom-val";

		// -- Exec
		let style = parse_style_str(style_str);

		// -- Check
		assert_eq!(style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(0x11, 0x22, 0x33))));
		assert_eq!(style.stroke, Some(SvgPaint::Url("g1".to_string())));
		assert_eq!(style.stroke_width, Some(3.5));
		assert_eq!(style.opacity, Some(0.8));
		assert_eq!(style.stroke_linecap, Some(StrokeLinecap::Round));
		assert_eq!(style.stroke_linejoin, Some(StrokeLinejoin::Bevel));
		assert_eq!(style.fill_rule, Some(FillRule::EvenOdd));
		assert_eq!(style.stroke_dasharray, Some(vec![5.0, 10.0, 15.0]));
		assert_eq!(
			style.extra.as_ref().and_then(|m| m.get("custom-prop")),
			Some(&"custom-val".to_string())
		);

		Ok(())
	}
}

// endregion: --- Tests
