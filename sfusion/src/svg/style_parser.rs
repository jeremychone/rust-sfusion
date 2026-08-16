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

	if s.starts_with("hsl(") || s.starts_with("hsla(") {
		return parse_hsl_fn(s);
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

fn parse_hsl_fn(s: &str) -> Option<SvgColor> {
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

	let h = parse_hue(parts[0])?;
	let s_val = parse_percentage_or_fraction(parts[1])?;
	let l_val = parse_percentage_or_fraction(parts[2])?;

	let a = if parts.len() >= 4 {
		parse_alpha_component(parts[3])?
	} else {
		1.0
	};

	let (r, g, b) = hsl_to_rgb(h, s_val, l_val);
	Some(SvgColor::new_rgba(r, g, b, a))
}

fn parse_hue(s: &str) -> Option<f64> {
	let s = s.trim();
	let s = s.strip_suffix("deg").unwrap_or(s);
	s.parse::<f64>().ok()
}

fn parse_percentage_or_fraction(s: &str) -> Option<f64> {
	let s = s.trim();
	if let Some(pct) = s.strip_suffix('%') {
		let num = pct.trim().parse::<f64>().ok()?;
		Some((num / 100.0).clamp(0.0, 1.0))
	} else {
		let num = s.parse::<f64>().ok()?;
		Some(num.clamp(0.0, 1.0))
	}
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
	let h = (h % 360.0 + 360.0) % 360.0;
	let s = s.clamp(0.0, 1.0);
	let l = l.clamp(0.0, 1.0);

	let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
	let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
	let m = l - c / 2.0;

	let (r1, g1, b1) = if h < 60.0 {
		(c, x, 0.0)
	} else if h < 120.0 {
		(x, c, 0.0)
	} else if h < 180.0 {
		(0.0, c, x)
	} else if h < 240.0 {
		(0.0, x, c)
	} else if h < 300.0 {
		(x, 0.0, c)
	} else {
		(c, 0.0, x)
	};

	(
		((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
		((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
		((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
	)
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
		"aliceblue" => (240, 248, 255),
		"antiquewhite" => (250, 235, 215),
		"aqua" | "cyan" => (0, 255, 255),
		"aquamarine" => (127, 255, 212),
		"azure" => (240, 255, 255),
		"beige" => (245, 245, 220),
		"bisque" => (255, 228, 196),
		"black" => (0, 0, 0),
		"blanchedalmond" => (255, 235, 205),
		"blue" => (0, 0, 255),
		"blueviolet" => (138, 43, 226),
		"brown" => (165, 42, 42),
		"burlywood" => (222, 184, 135),
		"cadetblue" => (95, 158, 160),
		"chartreuse" => (127, 255, 0),
		"chocolate" => (210, 105, 30),
		"coral" => (255, 127, 80),
		"cornflowerblue" => (100, 149, 237),
		"cornsilk" => (255, 248, 220),
		"crimson" => (220, 20, 60),
		"darkblue" => (0, 0, 139),
		"darkcyan" => (0, 139, 139),
		"darkgoldenrod" => (184, 134, 11),
		"darkgray" | "darkgrey" => (169, 169, 169),
		"darkgreen" => (0, 100, 0),
		"darkkhaki" => (189, 183, 107),
		"darkmagenta" => (139, 0, 139),
		"darkolivegreen" => (85, 107, 47),
		"darkorange" => (255, 140, 0),
		"darkorchid" => (153, 50, 204),
		"darkred" => (139, 0, 0),
		"darksalmon" => (233, 150, 122),
		"darkseagreen" => (143, 188, 143),
		"darkslateblue" => (72, 61, 139),
		"darkslategray" | "darkslategrey" => (47, 79, 79),
		"darkturquoise" => (0, 206, 209),
		"darkviolet" => (148, 0, 211),
		"deeppink" => (255, 20, 147),
		"deepskyblue" => (0, 191, 255),
		"dimgray" | "dimgrey" => (105, 105, 105),
		"dodgerblue" => (30, 144, 255),
		"firebrick" => (178, 34, 34),
		"floralwhite" => (255, 250, 240),
		"forestgreen" => (34, 139, 34),
		"fuchsia" | "magenta" => (255, 0, 255),
		"gainsboro" => (220, 220, 220),
		"ghostwhite" => (248, 248, 255),
		"gold" => (255, 215, 0),
		"goldenrod" => (218, 165, 32),
		"gray" | "grey" => (128, 128, 128),
		"green" => (0, 128, 0),
		"greenyellow" => (173, 255, 47),
		"honeydew" => (240, 255, 240),
		"hotpink" => (255, 105, 180),
		"indianred" => (205, 92, 92),
		"indigo" => (75, 0, 130),
		"ivory" => (255, 255, 240),
		"khaki" => (240, 230, 140),
		"lavender" => (230, 230, 250),
		"lavenderblush" => (255, 240, 245),
		"lawngreen" => (124, 252, 0),
		"lemonchiffon" => (255, 250, 205),
		"lightblue" => (173, 216, 230),
		"lightcoral" => (240, 128, 128),
		"lightcyan" => (224, 255, 255),
		"lightgoldenrodyellow" => (250, 250, 210),
		"lightgray" | "lightgrey" => (211, 211, 211),
		"lightgreen" => (144, 238, 144),
		"lightpink" => (255, 182, 193),
		"lightsalmon" => (255, 160, 122),
		"lightseagreen" => (32, 178, 170),
		"lightskyblue" => (135, 206, 250),
		"lightslategray" | "lightslategrey" => (119, 136, 153),
		"lightsteelblue" => (176, 196, 222),
		"lightyellow" => (255, 255, 224),
		"lime" => (0, 255, 0),
		"limegreen" => (50, 205, 50),
		"linen" => (250, 240, 230),
		"maroon" => (128, 0, 0),
		"mediumaquamarine" => (102, 205, 170),
		"mediumblue" => (0, 0, 205),
		"mediumorchid" => (186, 85, 211),
		"mediumpurple" => (147, 112, 219),
		"mediumseagreen" => (60, 179, 113),
		"mediumslateblue" => (123, 104, 238),
		"mediumspringgreen" => (0, 250, 154),
		"mediumturquoise" => (72, 209, 204),
		"mediumvioletred" => (199, 21, 133),
		"midnightblue" => (25, 25, 112),
		"mintcream" => (245, 255, 250),
		"mistyrose" => (255, 228, 225),
		"moccasin" => (255, 228, 181),
		"navajowhite" => (255, 222, 173),
		"navy" => (0, 0, 128),
		"oldlace" => (253, 245, 230),
		"olive" => (128, 128, 0),
		"olivedrab" => (107, 142, 35),
		"orange" => (255, 165, 0),
		"orangered" => (255, 69, 0),
		"orchid" => (218, 112, 214),
		"palegoldenrod" => (238, 232, 170),
		"palegreen" => (152, 251, 152),
		"paleturquoise" => (175, 238, 238),
		"palevioletred" => (219, 112, 147),
		"papayawhip" => (255, 239, 213),
		"peachpuff" => (255, 218, 185),
		"peru" => (205, 133, 63),
		"pink" => (255, 192, 203),
		"plum" => (221, 160, 221),
		"powderblue" => (176, 224, 230),
		"purple" => (128, 0, 128),
		"rebeccapurple" => (102, 51, 153),
		"red" => (255, 0, 0),
		"rosybrown" => (188, 143, 143),
		"royalblue" => (65, 105, 225),
		"saddlebrown" => (139, 69, 19),
		"salmon" => (250, 128, 114),
		"sandybrown" => (244, 164, 96),
		"seagreen" => (46, 139, 87),
		"seashell" => (255, 245, 238),
		"sienna" => (160, 82, 45),
		"silver" => (192, 192, 192),
		"skyblue" => (135, 206, 235),
		"slateblue" => (106, 90, 205),
		"slategray" | "slategrey" => (112, 128, 144),
		"snow" => (255, 250, 250),
		"springgreen" => (0, 255, 127),
		"steelblue" => (70, 130, 180),
		"tan" => (210, 180, 140),
		"teal" => (0, 128, 128),
		"thistle" => (216, 191, 216),
		"tomato" => (255, 99, 71),
		"turquoise" => (64, 224, 208),
		"violet" => (238, 130, 238),
		"wheat" => (245, 222, 179),
		"white" => (255, 255, 255),
		"whitesmoke" => (245, 245, 245),
		"yellow" => (255, 255, 0),
		"yellowgreen" => (154, 205, 50),
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
		let red_hsl = parse_color("hsl(0, 100%, 50%)").ok_or("Failed parsing hsl")?;
		let red_hsla = parse_color("hsla(0, 100%, 50%, 0.75)").ok_or("Failed parsing hsla")?;

		// -- Check
		assert_eq!(red_hex, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_short, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_name, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_rgb, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(red_rgba, SvgColor::new_rgba(255, 0, 0, 0.5));
		assert_eq!(red_hsl, SvgColor::new_rgba(255, 0, 0, 1.0));
		assert_eq!(red_hsla, SvgColor::new_rgba(255, 0, 0, 0.75));

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
