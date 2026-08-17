use super::naming::NameTracker;
use super::polyline::segments_to_polylines;
use crate::ast::*;
use crate::error::Result;
use crate::svg::{element_to_segments, parse_color, NormalizedSegment, Point};

// Layout grid constants for DaVinci Resolve Fusion operator spacing
const DEFAULT_START_X: f64 = 1980.0;
const DEFAULT_START_Y: f64 = -247.5;
const GRID_STEP_X: f64 = 110.0;
const GRID_STEP_Y: f64 = 66.0;

// region:    --- Types

#[derive(Default)]
pub struct GraphBuilder {
	name_tracker: NameTracker,
	tools: Vec<FusionTool>,
	col_counter: usize,
}

// endregion: --- Types

// region:    --- Public Functions

/// Converts an `SvgDoc` into a `FusionDoc` graph with positioned tools and merges.
pub fn build_fusion_doc(svg_doc: &SvgDoc) -> Result<FusionDoc> {
	let mut builder = GraphBuilder::default();
	let view_box = svg_doc.effective_view_box();

	let mut top_output_names = Vec::new();

	for element in &svg_doc.elements {
		let names = builder.process_element(element, &view_box, Transform2D::identity(), &SvgStyle::default())?;
		top_output_names.extend(names);
	}

	// Sort sPolygon tools alphabetically, followed by sMerge tools
	builder.tools.sort_by(|a, b| {
		let rank = |t: &FusionTool| match t {
			FusionTool::SPolygon(_) => 0,
			FusionTool::SText(_) => 1,
			FusionTool::SMerge(_) => 2,
		};
		let r_a = rank(a);
		let r_b = rank(b);
		if r_a != r_b {
			r_a.cmp(&r_b)
		} else {
			let name_a = match a {
				FusionTool::SPolygon(p) => &p.name,
				FusionTool::SText(t) => &t.name,
				FusionTool::SMerge(m) => &m.name,
			};
			let name_b = match b {
				FusionTool::SPolygon(p) => &p.name,
				FusionTool::SText(t) => &t.name,
				FusionTool::SMerge(m) => &m.name,
			};
			name_a.cmp(name_b)
		}
	});

	// If there are multiple top-level elements without a containing group, merge them
	if top_output_names.len() > 1 {
		let merge_name = builder.name_tracker.generate_unique_name(Some("smerge"));
		let pos = builder.next_merge_pos();
		let s_merge = SMerge {
			name: merge_name,
			inputs: top_output_names,
			view_info: pos,
		};
		builder.tools.push(FusionTool::SMerge(s_merge));
	}

	Ok(FusionDoc { tools: builder.tools })
}

// endregion: --- Public Functions

// region:    --- Support

impl GraphBuilder {
	fn next_leaf_pos(&mut self) -> ViewInfo {
		let pos_x = DEFAULT_START_X + (self.col_counter as f64) * GRID_STEP_X;
		let pos_y = DEFAULT_START_Y;
		self.col_counter += 1;
		ViewInfo::new(pos_x, pos_y)
	}

	fn next_merge_pos(&mut self) -> ViewInfo {
		let pos_x = DEFAULT_START_X + ((self.col_counter.saturating_sub(1)) as f64) * GRID_STEP_X;
		let pos_y = DEFAULT_START_Y + GRID_STEP_Y;
		ViewInfo::new(pos_x, pos_y)
	}

	fn process_element(
		&mut self,
		element: &SvgElement,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Vec<String>> {
		match element {
			SvgElement::Group(group) => self.process_group(group, view_box, parent_tf, parent_style),
			SvgElement::Text(text) => self.process_text(text, view_box, parent_tf, parent_style),
			_ => self.process_shape(element, view_box, parent_tf, parent_style),
		}
	}

	fn process_shape(
		&mut self,
		element: &SvgElement,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Vec<String>> {
		let elem_tf = get_element_transform(element).unwrap_or_default();
		let total_tf = parent_tf.multiply(&elem_tf);

		let raw_segments = element_to_segments(element)?;
		let transformed_segments: Vec<NormalizedSegment> = raw_segments
			.into_iter()
			.map(|seg| transform_segment(seg, total_tf))
			.collect();

		let polylines = segments_to_polylines(&transformed_segments, view_box);
		if polylines.is_empty() {
			return Ok(Vec::new());
		}

		let explicit_id = get_element_id(element);
		let effective_style = element.style().inherit_from(parent_style);
		let border_width = if effective_style.stroke.as_ref().is_some_and(|s| *s != SvgPaint::None) {
			let sw = effective_style.stroke_width.unwrap_or(1.0);
			let max_dim = view_box.width.max(view_box.height);
			let denom = if max_dim == 0.0 { 1.0 } else { max_dim };
			Some(sw / denom)
		} else {
			effective_style.stroke_width.map(|sw| {
				let max_dim = view_box.width.max(view_box.height);
				let denom = if max_dim == 0.0 { 1.0 } else { max_dim };
				sw / denom
			})
		};
		let (red, green, blue, opacity) = resolve_color_and_opacity(element.style(), &effective_style);
		let (mask_width, mask_height) = view_box.scaled_1080p_dimensions();
		if polylines.len() > 1 {
			let merge_name = self.name_tracker.generate_unique_name(explicit_id.or(Some("smerge")));
			let mut child_names = Vec::with_capacity(polylines.len());

			for poly in polylines {
				let poly_name = self.name_tracker.generate_unique_name(explicit_id.or(Some("poly")));
				let pos = self.next_leaf_pos();

				let spolygon = SPolygon {
					name: poly_name.clone(),
					mask_width,
					mask_height,
					border_width,
					red,
					green,
					blue,
					opacity,
					points: poly.points,
					closed: poly.closed,
					view_info: pos,
				};

				self.tools.push(FusionTool::SPolygon(spolygon));
				child_names.push(poly_name);
			}

			let merge_pos = self.next_merge_pos();
			let s_merge = SMerge {
				name: merge_name.clone(),
				inputs: child_names,
				view_info: merge_pos,
			};
			self.tools.push(FusionTool::SMerge(s_merge));

			Ok(vec![merge_name])
		} else {
			let mut shape_tool_names = Vec::new();
			for poly in polylines {
				let name = self.name_tracker.generate_unique_name(explicit_id);
				let pos = self.next_leaf_pos();

				let spolygon = SPolygon {
					name: name.clone(),
					mask_width,
					mask_height,
					border_width,
					red,
					green,
					blue,
					opacity,
					points: poly.points,
					closed: poly.closed,
					view_info: pos,
				};

				self.tools.push(FusionTool::SPolygon(spolygon));
				shape_tool_names.push(name);
			}

			Ok(shape_tool_names)
		}
	}

	fn process_group(
		&mut self,
		group: &SvgGroup,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Vec<String>> {
		let group_tf = group.transform.unwrap_or_default();
		let total_tf = parent_tf.multiply(&group_tf);
		let effective_style = group.style.inherit_from(parent_style);

		let mut child_names = Vec::new();

		for child in &group.children {
			let names = self.process_element(child, view_box, total_tf, &effective_style)?;
			child_names.extend(names);
		}

		if child_names.is_empty() {
			return Ok(Vec::new());
		}

		if child_names.len() == 1 {
			let mut child_name = child_names.remove(0);
			if let Some(group_id) = group.id.as_deref()
				&& !group_id.trim().is_empty()
			{
				let new_name = self.name_tracker.generate_unique_name(Some(group_id));
				for tool in &mut self.tools {
					match tool {
						FusionTool::SPolygon(poly) if poly.name == child_name => {
							poly.name = new_name.clone();
						}
						FusionTool::SText(text) if text.name == child_name => {
							text.name = new_name.clone();
						}
						FusionTool::SMerge(merge) if merge.name == child_name => {
							merge.name = new_name.clone();
						}
						_ => {}
					}
					if let FusionTool::SMerge(merge) = tool {
						for input in &mut merge.inputs {
							if *input == child_name {
								*input = new_name.clone();
							}
						}
					}
				}
				child_name = new_name;
			}
			return Ok(vec![child_name]);
		}

		let group_id = group.id.as_deref().or(Some("smerge"));
		let merge_name = self.name_tracker.generate_unique_name(group_id);
		let pos = self.next_merge_pos();

		let s_merge = SMerge {
			name: merge_name.clone(),
			inputs: child_names,
			view_info: pos,
		};

		self.tools.push(FusionTool::SMerge(s_merge));
		Ok(vec![merge_name])
	}

	fn process_text(
		&mut self,
		text: &SvgText,
		_view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Vec<String>> {
		let elem_tf = text.transform.unwrap_or_default();
		let _total_tf = parent_tf.multiply(&elem_tf);

		let explicit_id = text.id.as_deref();
		let effective_style = text.style.inherit_from(parent_style);

		let name = self.name_tracker.generate_unique_name(explicit_id.or(Some("stext")));
		let pos = self.next_leaf_pos();

		let styled_text = text.content.trim().to_string();

		let font_family = text
			.font_family
			.as_deref()
			.or_else(|| effective_style.extra.as_ref().and_then(|m| m.get("font-family").map(|s| s.as_str())));

		let font = font_family
			.and_then(clean_font_family);

		let font_weight = text
			.font_weight
			.as_deref()
			.or_else(|| effective_style.extra.as_ref().and_then(|m| m.get("font-weight").map(|s| s.as_str())));

		let font_style = text
			.font_style
			.as_deref()
			.or_else(|| effective_style.extra.as_ref().and_then(|m| m.get("font-style").map(|s| s.as_str())));

		let style = map_font_style(font_weight, font_style);

		let (red, green, blue, opacity) = resolve_color_and_opacity(&text.style, &effective_style);

		let text_anchor = text
			.text_anchor
			.as_deref()
			.or_else(|| effective_style.extra.as_ref().and_then(|m| m.get("text-anchor").map(|s| s.as_str())));

		let (h_just, h_lcr) = match text_anchor.map(|a| a.trim().to_lowercase()).as_deref() {
			Some("middle") | Some("center") => (Some(3), Some(1)),
			Some("end") | Some("right") => (Some(2), Some(2)),
			Some("start") | Some("left") => (Some(0), Some(0)),
			_ => (Some(3), None),
		};

		let stext = SText {
			name: name.clone(),
			styled_text,
			font,
			style,
			line_spacing: None,
			character_spacing: None,
			red,
			green,
			blue,
			opacity,
			vertical_justification: Some(3),
			horizontal_justification: h_just,
			horizontal_left_center_right: h_lcr,
			wrap: Some(1),
			layout_rotation: Some(1),
			transform_rotation: Some(1),
			center_x: None,
			center_y: None,
			view_info: pos,
		};

		self.tools.push(FusionTool::SText(stext));
		Ok(vec![name])
	}
}

fn clean_font_family(raw_font: &str) -> Option<String> {
	let candidates = raw_font.split(',');
	let mut generic_fallback = None;

	for candidate in candidates {
		let trimmed = candidate.trim().trim_matches('\'').trim_matches('"').trim();
		if trimmed.is_empty() {
			continue;
		}

		let is_generic = matches!(
			trimmed.to_ascii_lowercase().as_str(),
			"sans-serif"
				| "serif"
				| "monospace"
				| "cursive"
				| "fantasy"
				| "system-ui"
				| "ui-sans-serif"
				| "ui-serif"
				| "ui-monospace"
		);

		if is_generic {
			if generic_fallback.is_none() {
				generic_fallback = Some(trimmed.to_string());
			}
			continue;
		}

		let cleaned = sanitize_font_name(trimmed);
		if !cleaned.is_empty() {
			return Some(cleaned);
		}
	}

	generic_fallback
}

fn sanitize_font_name(name: &str) -> String {
	let mut s = name.trim();

	if let Some(idx) = s.to_ascii_lowercase().find("-variablefont") {
		s = &s[..idx];
	} else if let Some(idx) = s.to_ascii_lowercase().find("_variablefont") {
		s = &s[..idx];
	}

	const KNOWN_SUFFIXES: &[&str] = &[
		"bolditalic",
		"bold_italic",
		"bold italic",
		"semibolditalic",
		"semibold_italic",
		"semibold italic",
		"extrabolditalic",
		"extrabold_italic",
		"extrabold italic",
		"mediumitalic",
		"medium_italic",
		"medium italic",
		"lightitalic",
		"light_italic",
		"light italic",
		"thinitalic",
		"thin_italic",
		"thin italic",
		"extralightitalic",
		"extralight_italic",
		"extralight italic",
		"ultralightitalic",
		"ultralight_italic",
		"ultralight italic",
		"semibold",
		"semi_bold",
		"semi bold",
		"demibold",
		"demi_bold",
		"demi bold",
		"extrabold",
		"extra_bold",
		"extra bold",
		"ultrabold",
		"ultra_bold",
		"ultra bold",
		"extralight",
		"extra_light",
		"extra light",
		"ultralight",
		"ultra_light",
		"ultra light",
		"regular",
		"medium",
		"italic",
		"oblique",
		"bold",
		"light",
		"thin",
		"heavy",
		"black",
		"book",
		"demi",
	];

	let lower = s.to_ascii_lowercase();
	for suffix in KNOWN_SUFFIXES {
		if lower.ends_with(suffix) {
			let prefix_len = s.len() - suffix.len();
			let prefix = &s[..prefix_len];
			if prefix.ends_with('-') || prefix.ends_with('_') || prefix.ends_with(' ') {
				let trimmed_prefix = prefix.trim_end_matches(['-', '_', ' ']);
				if !trimmed_prefix.is_empty() {
					s = trimmed_prefix;
					break;
				}
			}
		}
	}

	s.to_string()
}

fn capitalize_words(s: &str) -> String {
	s.split_whitespace()
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
				None => String::new(),
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}

fn map_font_style(weight: Option<&str>, style: Option<&str>) -> Option<String> {
	let w = weight.map(|s| s.trim().to_lowercase());
	let s = style.map(|s| s.trim().to_lowercase());

	let is_italic = matches!(s.as_deref(), Some("italic") | Some("oblique"));

	let base_weight = match w.as_deref() {
		Some("100") | Some("thin") | Some("hairline") => Some("Thin"),
		Some("200") | Some("extralight") | Some("extra-light") | Some("extra light") | Some("ultralight") | Some("ultra-light") | Some("ultra light") => Some("ExtraLight"),
		Some("300") | Some("light") => Some("Light"),
		Some("400") | Some("normal") | Some("regular") | Some("book") => Some("Regular"),
		Some("500") | Some("medium") => Some("Medium"),
		Some("600") | Some("semibold") | Some("semi-bold") | Some("semi bold") | Some("demibold") | Some("demi-bold") | Some("demi bold") => Some("SemiBold"),
		Some("700") | Some("bold") | Some("bolder") => Some("Bold"),
		Some("800") | Some("extrabold") | Some("extra-bold") | Some("extra bold") | Some("ultrabold") | Some("ultra-bold") | Some("ultra bold") => Some("ExtraBold"),
		Some("900") | Some("black") | Some("heavy") => Some("Black"),
		_ => None,
	};

	match (base_weight, is_italic) {
		(Some("Regular"), true) | (None, true) => {
			if base_weight.is_none() && let Some(w_str) = &w && !w_str.is_empty() {
				let cap_weight = capitalize_words(w_str);
				Some(format!("{cap_weight} Italic"))
			} else {
				Some("Italic".to_string())
			}
		}
		(Some("Regular"), false) => Some("Regular".to_string()),
		(Some(bw), true) => Some(format!("{bw} Italic")),
		(Some(bw), false) => Some(bw.to_string()),
		(None, false) => {
			if let Some(w_str) = &w && !w_str.is_empty() {
				Some(capitalize_words(w_str))
			} else if let Some(s_str) = &s && !s_str.is_empty() && s_str != "normal" {
				Some(capitalize_words(s_str))
			} else {
				None
			}
		}
	}
}

fn transform_segment(seg: NormalizedSegment, tf: Transform2D) -> NormalizedSegment {
	match seg {
		NormalizedSegment::MoveTo(p) => {
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::MoveTo(Point::new(x, y))
		}
		NormalizedSegment::LineTo(p) => {
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::LineTo(Point::new(x, y))
		}
		NormalizedSegment::CubicTo { p1, p2, p } => {
			let (x1, y1) = tf.transform_xy(p1.x, p1.y);
			let (x2, y2) = tf.transform_xy(p2.x, p2.y);
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::CubicTo {
				p1: Point::new(x1, y1),
				p2: Point::new(x2, y2),
				p: Point::new(x, y),
			}
		}
		NormalizedSegment::Close => NormalizedSegment::Close,
	}
}

fn get_element_transform(element: &SvgElement) -> Option<Transform2D> {
	match element {
		SvgElement::Path(p) => p.transform,
		SvgElement::Rect(r) => r.transform,
		SvgElement::Circle(c) => c.transform,
		SvgElement::Ellipse(e) => e.transform,
		SvgElement::Line(l) => l.transform,
		SvgElement::Polyline(pl) => pl.transform,
		SvgElement::Polygon(pg) => pg.transform,
		SvgElement::Group(g) => g.transform,
		SvgElement::Text(t) => t.transform,
	}
}

fn get_element_id(element: &SvgElement) -> Option<&str> {
	match element {
		SvgElement::Path(p) => p.id.as_deref(),
		SvgElement::Rect(r) => r.id.as_deref(),
		SvgElement::Circle(c) => c.id.as_deref(),
		SvgElement::Ellipse(e) => e.id.as_deref(),
		SvgElement::Line(l) => l.id.as_deref(),
		SvgElement::Polyline(pl) => pl.id.as_deref(),
		SvgElement::Polygon(pg) => pg.id.as_deref(),
		SvgElement::Group(g) => g.id.as_deref(),
		SvgElement::Text(t) => t.id.as_deref(),
	}
}

fn resolve_color_and_opacity(
	element_style: &SvgStyle,
	effective_style: &SvgStyle,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
	let resolve_paint = |paint: &Option<SvgPaint>| -> Option<SvgColor> {
		match paint {
			Some(SvgPaint::Color(c)) => Some(c.clone()),
			Some(SvgPaint::CurrentColor) => {
				let color_val = effective_style
					.extra
					.as_ref()
					.and_then(|m| m.get("color"))
					.and_then(|c| parse_color(c));
				Some(color_val.unwrap_or_else(|| SvgColor::new_rgb(0, 0, 0)))
			}
			_ => None,
		}
	};

	let child_has_stroke = element_style
		.stroke
		.as_ref()
		.is_some_and(|s| *s != SvgPaint::None);
	let child_has_fill = element_style.fill.is_some();

	let (paint_color, is_stroke) = if child_has_stroke && !child_has_fill {
		(resolve_paint(&effective_style.stroke), true)
	} else {
		match (&effective_style.fill, &effective_style.stroke) {
			(Some(SvgPaint::Color(_)), _) | (Some(SvgPaint::CurrentColor), _) => {
				(resolve_paint(&effective_style.fill), false)
			}
			(Some(SvgPaint::None), Some(stroke)) if *stroke != SvgPaint::None => {
				(resolve_paint(&effective_style.stroke), true)
			}
			(None, Some(stroke)) if *stroke != SvgPaint::None => {
				(resolve_paint(&effective_style.stroke), true)
			}
			(Some(SvgPaint::None), _) => (None, false),
			(None, None) | (None, Some(SvgPaint::None)) => {
				(Some(SvgColor::new_rgb(0, 0, 0)), false)
			}
			_ => (None, false),
		}
	};

	let (red, green, blue, color_alpha) = if let Some(c) = paint_color {
		(
			Some(c.r as f64 / 255.0),
			Some(c.g as f64 / 255.0),
			Some(c.b as f64 / 255.0),
			if (c.a - 1.0).abs() > 1e-6 { Some(c.a) } else { None },
		)
	} else {
		(None, None, None, None)
	};

	let mut effective_op = effective_style.opacity;
	let specific_op = if is_stroke {
		effective_style.stroke_opacity
	} else {
		effective_style.fill_opacity
	};

	if let Some(spec_op) = specific_op {
		effective_op = Some(effective_op.unwrap_or(1.0) * spec_op);
	}

	if let Some(alpha) = color_alpha {
		effective_op = Some(effective_op.unwrap_or(1.0) * alpha);
	}

	(red, green, blue, effective_op)
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_fusion_graph_builder_two_shapes() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 320.0, 240.0)),
			width: Some(320.0),
			height: Some(240.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Path(SvgPath {
					id: Some("poly_1".to_string()),
					transform: None,
					style: SvgStyle::default(),
					d: "M 10 20 L 30 40 Z".to_string(),
				}),
				SvgElement::Rect(SvgRect {
					id: Some("grabber".to_string()),
					transform: None,
					style: SvgStyle::default(),
					x: 10.0,
					y: 20.0,
					width: 50.0,
					height: 60.0,
					rx: None,
					ry: None,
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);

		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "grabber");
			assert_eq!(p1.mask_width, 1080.0);
			assert_eq!(p1.mask_height, 810.0);
			assert_eq!(p1.view_info.pos_x, 2090.0);
			assert_eq!(p1.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon grabber as first tool".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "poly_1");
			assert_eq!(p2.mask_width, 1080.0);
			assert_eq!(p2.mask_height, 810.0);
			assert_eq!(p2.view_info.pos_x, 1980.0);
			assert_eq!(p2.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon poly_1 as second tool".into());
		}

		if let FusionTool::SMerge(m) = &fusion_doc.tools[2] {
			assert_eq!(m.name, "smerge");
			assert_eq!(m.inputs, vec!["poly_1", "grabber"]);
			assert_eq!(m.view_info.pos_x, 2090.0);
			assert_eq!(m.view_info.pos_y, -181.5);
		} else {
			return Err("Expected SMerge as third tool".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_single_child_group_inherits_id() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 320.0, 240.0)),
			width: Some(320.0),
			height: Some(240.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("grabber".to_string()),
				transform: None,
				style: SvgStyle::default(),
				children: vec![SvgElement::Circle(SvgCircle {
					id: None,
					transform: None,
					style: SvgStyle::default(),
					cx: 50.0,
					cy: 50.0,
					r: 25.0,
				})],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 1);
		if let FusionTool::SPolygon(p) = &fusion_doc.tools[0] {
			assert_eq!(p.name, "grabber");
			assert_eq!(p.border_width, None);
		} else {
			return Err("Expected SPolygon grabber".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_stroke_width_inheritance() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 200.0, 100.0)),
			width: Some(200.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("styled_group".to_string()),
				transform: None,
				style: SvgStyle {
					stroke_width: Some(4.0),
					..Default::default()
				},
				children: vec![
					SvgElement::Path(SvgPath {
						id: Some("inherited_path".to_string()),
						transform: None,
						style: SvgStyle::default(),
						d: "M 0 0 L 10 10".to_string(),
					}),
					SvgElement::Path(SvgPath {
						id: Some("override_path".to_string()),
						transform: None,
						style: SvgStyle {
							stroke_width: Some(10.0),
							..Default::default()
						},
						d: "M 10 10 L 20 20".to_string(),
					}),
				],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "inherited_path");
			assert_eq!(p1.border_width, Some(4.0 / 200.0));
		} else {
			return Err("Expected inherited_path SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "override_path");
			assert_eq!(p2.border_width, Some(10.0 / 200.0));
		} else {
			return Err("Expected override_path SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_deep_nested_group_styles() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 400.0, 200.0)),
			width: Some(400.0),
			height: Some(200.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("root_group".to_string()),
				transform: None,
				style: SvgStyle {
					stroke_width: Some(8.0),
					..Default::default()
				},
				children: vec![SvgElement::Group(SvgGroup {
					id: Some("inner_group".to_string()),
					transform: None,
					style: SvgStyle::default(),
					children: vec![
						SvgElement::Rect(SvgRect {
							id: Some("rect1".to_string()),
							transform: None,
							style: SvgStyle::default(),
							x: 0.0,
							y: 0.0,
							width: 50.0,
							height: 50.0,
							rx: None,
							ry: None,
						}),
						SvgElement::Rect(SvgRect {
							id: Some("rect2".to_string()),
							transform: None,
							style: SvgStyle {
								stroke_width: Some(2.0),
								..Default::default()
							},
							x: 60.0,
							y: 0.0,
							width: 50.0,
							height: 50.0,
							rx: None,
							ry: None,
						}),
					],
				})],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "rect1");
			assert_eq!(p1.border_width, Some(8.0 / 400.0));
		} else {
			return Err("Expected rect1 SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "rect2");
			assert_eq!(p2.border_width, Some(2.0 / 400.0));
		} else {
			return Err("Expected rect2 SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_color_and_opacity_mapping() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Rect(SvgRect {
					id: Some("filled_rect".to_string()),
					transform: None,
					style: SvgStyle {
						fill: Some(SvgPaint::Color(SvgColor::new_rgba(255, 128, 0, 0.5))),
						opacity: Some(0.8),
						..Default::default()
					},
					x: 0.0,
					y: 0.0,
					width: 10.0,
					height: 10.0,
					rx: None,
					ry: None,
				}),
				SvgElement::Path(SvgPath {
					id: Some("stroked_path".to_string()),
					transform: None,
					style: SvgStyle {
						fill: Some(SvgPaint::None),
						stroke: Some(SvgPaint::Color(SvgColor::new_rgb(0, 0, 255))),
						stroke_width: Some(2.0),
						stroke_opacity: Some(0.75),
						..Default::default()
					},
					d: "M 0 0 L 10 10".to_string(),
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "filled_rect");
			assert_eq!(p1.red, Some(1.0));
			assert!((p1.green.ok_or("missing green")? - 128.0 / 255.0).abs() < 1e-6);
			assert_eq!(p1.blue, Some(0.0));
			// opacity = 0.8 * 0.5 = 0.4
			assert!((p1.opacity.ok_or("missing opacity")? - 0.4).abs() < 1e-6);
		} else {
			return Err("Expected filled_rect SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "stroked_path");
			assert_eq!(p2.red, Some(0.0));
			assert_eq!(p2.green, Some(0.0));
			assert_eq!(p2.blue, Some(1.0));
			assert_eq!(p2.opacity, Some(0.75));
			assert_eq!(p2.border_width, Some(0.02));
		} else {
			return Err("Expected stroked_path SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_hierarchical_transforms() -> Result<()> {
		// -- Setup & Fixtures
		// Root view box 200x200, center at (100, 100).
		// Outer group translates by (+50, +20).
		// Inner group scales by 2x.
		// Inner shape is a rect at (10, 10) with width 10, height 10.
		// Transformed rect in SVG space:
		// x: 50 + (10 * 2) = 70, y: 20 + (10 * 2) = 40.
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 200.0, 200.0)),
			width: Some(200.0),
			height: Some(200.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("outer_grp".to_string()),
				transform: Some(Transform2D::translate(50.0, 20.0)),
				style: SvgStyle::default(),
				children: vec![SvgElement::Group(SvgGroup {
					id: Some("inner_grp".to_string()),
					transform: Some(Transform2D::scale(2.0, 2.0)),
					style: SvgStyle::default(),
					children: vec![SvgElement::Rect(SvgRect {
						id: Some("box".to_string()),
						transform: None,
						style: SvgStyle::default(),
						x: 10.0,
						y: 10.0,
						width: 10.0,
						height: 10.0,
						rx: None,
						ry: None,
					})],
				})],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 1);
		if let FusionTool::SPolygon(p) = &fusion_doc.tools[0] {
			assert_eq!(p.name, "outer_grp");
			// First point should be transformed (70, 40)
			// Center is (100, 100).
			// rel_x = 70 - 100 = -30.
			// rel_y = 40 - 100 = -60.
			// nx = -30 / 200 = -0.15
			// ny = -(-60) / 200 = 0.3
			let p0 = p.points[0];
			assert!((p0.x - (-0.15)).abs() < 1e-6);
			assert!((p0.y - 0.3).abs() < 1e-6);
		} else {
			return Err("Expected SPolygon box".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_layer_ordering_three_elements() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Rect(SvgRect {
					id: Some("layer_bottom".to_string()),
					transform: None,
					style: SvgStyle::default(),
					x: 0.0,
					y: 0.0,
					width: 10.0,
					height: 10.0,
					rx: None,
					ry: None,
				}),
				SvgElement::Circle(SvgCircle {
					id: Some("layer_middle".to_string()),
					transform: None,
					style: SvgStyle::default(),
					cx: 20.0,
					cy: 20.0,
					r: 5.0,
				}),
				SvgElement::Path(SvgPath {
					id: Some("layer_top".to_string()),
					transform: None,
					style: SvgStyle::default(),
					d: "M 30 30 L 40 40".to_string(),
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		let merge_tool = fusion_doc
			.tools
			.iter()
			.find_map(|t| match t {
				FusionTool::SMerge(m) => Some(m),
				_ => None,
			})
			.ok_or("Expected SMerge tool")?;

		assert_eq!(
			merge_tool.inputs,
			vec!["layer_bottom", "layer_middle", "layer_top"]
		);

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_multi_polyline_shape_ordering() -> Result<()> {
		// -- Setup & Fixtures
		// Two disjoint sub-paths in a single SVG path
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Path(SvgPath {
				id: Some("multi_path".to_string()),
				transform: None,
				style: SvgStyle::default(),
				d: "M 0 0 L 10 10 M 20 20 L 30 30".to_string(),
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		let merge_tool = fusion_doc
			.tools
			.iter()
			.find_map(|t| match t {
				FusionTool::SMerge(m) => Some(m),
				_ => None,
			})
			.ok_or("Expected SMerge tool for multi-subpath shape")?;

		assert_eq!(merge_tool.inputs.len(), 2);
		assert_eq!(merge_tool.inputs[0], "multi_path_1");
		assert_eq!(merge_tool.inputs[1], "multi_path_2");

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_multi_polyline_inside_group_preserves_smerge() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("crabby_group".to_string()),
				transform: None,
				style: SvgStyle::default(),
				children: vec![
					SvgElement::Path(SvgPath {
						id: Some("body".to_string()),
						transform: None,
						style: SvgStyle::default(),
						d: "M 0 0 L 10 10 M 20 20 L 30 30".to_string(),
					}),
					SvgElement::Circle(SvgCircle {
						id: Some("eye".to_string()),
						transform: None,
						style: SvgStyle::default(),
						cx: 50.0,
						cy: 50.0,
						r: 5.0,
					}),
				],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		// Tools: body_1, body_2, eye (3 polygons) + body (sMerge), crabby_group (sMerge). Total: 5 tools
		assert_eq!(fusion_doc.tools.len(), 5);
		let merge_tools: Vec<&SMerge> = fusion_doc
			.tools
			.iter()
			.filter_map(|t| match t {
				FusionTool::SMerge(m) => Some(m),
				_ => None,
			})
			.collect();

		assert_eq!(merge_tools.len(), 2);
		let body_merge = merge_tools.iter().find(|m| m.name == "body").ok_or("missing body merge")?;
		let group_merge = merge_tools.iter().find(|m| m.name == "crabby_group").ok_or("missing group merge")?;
		assert_eq!(body_merge.inputs, vec!["body_1", "body_2"]);
		assert_eq!(group_merge.inputs, vec!["body", "eye"]);

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_current_color_and_hsl() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Group(SvgGroup {
					id: Some("color_group".to_string()),
					transform: None,
					style: SvgStyle {
						extra: Some({
							let mut map = std::collections::HashMap::new();
							map.insert("color".to_string(), "#00ff00".to_string());
							map
						}),
						..Default::default()
					},
					children: vec![SvgElement::Rect(SvgRect {
						id: Some("current_color_rect".to_string()),
						transform: None,
						style: SvgStyle {
							fill: Some(SvgPaint::CurrentColor),
							..Default::default()
						},
						x: 0.0,
						y: 0.0,
						width: 10.0,
						height: 10.0,
						rx: None,
						ry: None,
					})],
				}),
				SvgElement::Path(SvgPath {
					id: Some("hsl_path".to_string()),
					transform: None,
					style: SvgStyle {
						fill: Some(SvgPaint::None),
						stroke: Some(SvgPaint::Color(SvgColor::new_rgba(255, 0, 0, 0.5))),
						stroke_width: Some(2.0),
						..Default::default()
					},
					d: "M 0 0 L 10 10".to_string(),
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "color_group");
			assert_eq!(p1.red, Some(0.0));
			assert_eq!(p1.green, Some(1.0));
			assert_eq!(p1.blue, Some(0.0));
		} else {
			return Err("Expected color_group SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "hsl_path");
			assert_eq!(p2.red, Some(1.0));
			assert_eq!(p2.green, Some(0.0));
			assert_eq!(p2.blue, Some(0.0));
			assert_eq!(p2.opacity, Some(0.5));
		} else {
			return Err("Expected hsl_path SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_default_fill_and_stroke_priority() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 100.0, 100.0)),
			width: Some(100.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Rect(SvgRect {
					id: Some("black_default".to_string()),
					transform: None,
					style: SvgStyle::default(),
					x: 0.0,
					y: 0.0,
					width: 10.0,
					height: 10.0,
					rx: None,
					ry: None,
				}),
				SvgElement::Group(SvgGroup {
					id: Some("parent_grp".to_string()),
					transform: None,
					style: SvgStyle {
						fill: Some(SvgPaint::Color(SvgColor::new_rgb(255, 128, 0))),
						..Default::default()
					},
					children: vec![SvgElement::Path(SvgPath {
						id: Some("white_stroke".to_string()),
						transform: None,
						style: SvgStyle {
							stroke: Some(SvgPaint::Color(SvgColor::new_rgb(255, 255, 255))),
							stroke_width: Some(2.0),
							..Default::default()
						},
						d: "M 10 10 L 20 20".to_string(),
					})],
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "black_default");
			assert_eq!(p1.red, Some(0.0));
			assert_eq!(p1.green, Some(0.0));
			assert_eq!(p1.blue, Some(0.0));
		} else {
			return Err("Expected black_default SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "parent_grp");
			assert_eq!(p2.red, Some(1.0));
			assert_eq!(p2.green, Some(1.0));
			assert_eq!(p2.blue, Some(1.0));
		} else {
			return Err("Expected parent_grp SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_open_curve_and_border_width() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 200.0, 200.0)),
			width: Some(200.0),
			height: Some(200.0),
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Path(SvgPath {
					id: Some("open_stroke".to_string()),
					transform: None,
					style: SvgStyle {
						stroke: Some(SvgPaint::Color(SvgColor::new_rgb(255, 255, 255))),
						..Default::default()
					},
					d: "M 10 20 C 30 40 50 60 70 80".to_string(),
				}),
				SvgElement::Path(SvgPath {
					id: Some("closed_poly".to_string()),
					transform: None,
					style: SvgStyle::default(),
					d: "M 10 20 L 30 40 L 50 20 Z".to_string(),
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "closed_poly");
			assert!(p1.closed);
		} else {
			return Err("Expected closed_poly SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "open_stroke");
			assert!(!p2.closed);
			// Default stroke width = 1.0 / 200.0 = 0.005
			assert_eq!(p2.border_width, Some(1.0 / 200.0));
			assert_eq!(p2.red, Some(1.0));
			assert_eq!(p2.green, Some(1.0));
			assert_eq!(p2.blue, Some(1.0));
		} else {
			return Err("Expected open_stroke SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_stext_simple() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 400.0, 300.0)),
			width: Some(400.0),
			height: Some(300.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Text(SvgText {
				id: Some("heading_txt".to_string()),
				transform: None,
				style: SvgStyle {
					fill: Some(SvgPaint::Color(SvgColor::new_rgb(255, 128, 0))),
					..Default::default()
				},
				x: Some(50.0),
				y: Some(50.0),
				dx: None,
				dy: None,
				font_family: Some("Roboto".to_string()),
				font_size: Some(24.0),
				font_weight: Some("bold".to_string()),
				font_style: None,
				text_anchor: Some("middle".to_string()),
				content: "Hello World".to_string(),
				children: Vec::new(),
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 1);
		if let FusionTool::SText(txt) = &fusion_doc.tools[0] {
			assert_eq!(txt.name, "heading_txt");
			assert_eq!(txt.styled_text, "Hello World");
			assert_eq!(txt.font.as_deref(), Some("Roboto"));
			assert_eq!(txt.style.as_deref(), Some("Bold"));
			assert_eq!(txt.red, Some(1.0));
			assert!((txt.green.ok_or("missing green")? - 128.0 / 255.0).abs() < 1e-6);
			assert_eq!(txt.blue, Some(0.0));
			assert_eq!(txt.horizontal_justification, Some(3));
			assert_eq!(txt.horizontal_left_center_right, Some(1));
		} else {
			return Err("Expected SText tool".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_stext_in_group_with_shapes() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 500.0, 500.0)),
			width: Some(500.0),
			height: Some(500.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("card_group".to_string()),
				transform: None,
				style: SvgStyle::default(),
				children: vec![
					SvgElement::Rect(SvgRect {
						id: Some("bg_box".to_string()),
						transform: None,
						style: SvgStyle::default(),
						x: 0.0,
						y: 0.0,
						width: 200.0,
						height: 100.0,
						rx: None,
						ry: None,
					}),
					SvgElement::Text(SvgText {
						id: Some("label".to_string()),
						transform: None,
						style: SvgStyle::default(),
						x: Some(10.0),
						y: Some(20.0),
						dx: None,
						dy: None,
						font_family: Some("'Open Sans'".to_string()),
						font_size: Some(14.0),
						font_weight: Some("700".to_string()),
						font_style: Some("italic".to_string()),
						text_anchor: Some("start".to_string()),
						content: "Card Title".to_string(),
						children: Vec::new(),
					}),
				],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		let polygon_tool = fusion_doc.tools.iter().find(|t| matches!(t, FusionTool::SPolygon(_)));
		let text_tool = fusion_doc.tools.iter().find(|t| matches!(t, FusionTool::SText(_)));
		let merge_tool = fusion_doc.tools.iter().find(|t| matches!(t, FusionTool::SMerge(_)));

		assert!(polygon_tool.is_some());
		assert!(text_tool.is_some());
		assert!(merge_tool.is_some());

		if let Some(FusionTool::SText(txt)) = text_tool {
			assert_eq!(txt.name, "label");
			assert_eq!(txt.styled_text, "Card Title");
			assert_eq!(txt.font.as_deref(), Some("Open Sans"));
			assert_eq!(txt.style.as_deref(), Some("Bold Italic"));
			assert_eq!(txt.horizontal_justification, Some(0));
			assert_eq!(txt.horizontal_left_center_right, Some(0));
		}

		if let Some(FusionTool::SMerge(m)) = merge_tool {
			assert_eq!(m.name, "card_group");
			assert_eq!(m.inputs, vec!["bg_box", "label"]);
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_clean_font_family() -> Result<()> {
		// -- Exec & Check
		assert_eq!(
			clean_font_family("Lato-Bold, Lato, sans-serif").as_deref(),
			Some("Lato")
		);
		assert_eq!(
			clean_font_family("'Open Sans-SemiBold', 'Open Sans', sans-serif").as_deref(),
			Some("Open Sans")
		);
		assert_eq!(
			clean_font_family("Roboto-Regular").as_deref(),
			Some("Roboto")
		);
		assert_eq!(
			clean_font_family("'Montserrat-ExtraBold'").as_deref(),
			Some("Montserrat")
		);
		assert_eq!(
			clean_font_family("Fira Code, monospace").as_deref(),
			Some("Fira Code")
		);
		assert_eq!(
			clean_font_family("sans-serif").as_deref(),
			Some("sans-serif")
		);
		assert_eq!(
			clean_font_family("Inter-VariableFont_opsz,wght").as_deref(),
			Some("Inter")
		);
		assert_eq!(clean_font_family("   ").as_deref(), None);

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_map_font_style() -> Result<()> {
		// -- Exec & Check
		assert_eq!(map_font_style(Some("700"), None).as_deref(), Some("Bold"));
		assert_eq!(map_font_style(Some("bold"), None).as_deref(), Some("Bold"));
		assert_eq!(map_font_style(Some("700"), Some("italic")).as_deref(), Some("Bold Italic"));
		assert_eq!(map_font_style(Some("400"), None).as_deref(), Some("Regular"));
		assert_eq!(map_font_style(Some("normal"), None).as_deref(), Some("Regular"));
		assert_eq!(map_font_style(Some("400"), Some("italic")).as_deref(), Some("Italic"));
		assert_eq!(map_font_style(None, Some("italic")).as_deref(), Some("Italic"));
		assert_eq!(map_font_style(Some("300"), None).as_deref(), Some("Light"));
		assert_eq!(map_font_style(Some("light"), Some("italic")).as_deref(), Some("Light Italic"));
		assert_eq!(map_font_style(Some("100"), None).as_deref(), Some("Thin"));
		assert_eq!(map_font_style(Some("200"), None).as_deref(), Some("ExtraLight"));
		assert_eq!(map_font_style(Some("500"), None).as_deref(), Some("Medium"));
		assert_eq!(map_font_style(Some("600"), None).as_deref(), Some("SemiBold"));
		assert_eq!(map_font_style(Some("800"), None).as_deref(), Some("ExtraBold"));
		assert_eq!(map_font_style(Some("900"), None).as_deref(), Some("Black"));
		assert_eq!(map_font_style(Some("black"), Some("italic")).as_deref(), Some("Black Italic"));
		assert_eq!(map_font_style(None, None), None);

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_nested_tspan_flattening() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 500.0, 200.0)),
			width: Some(500.0),
			height: Some(200.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Text(SvgText {
				id: Some("brand".to_string()),
				transform: None,
				style: SvgStyle::default(),
				x: Some(50.0),
				y: Some(100.0),
				dx: None,
				dy: None,
				font_family: Some("Lato-Bold, Lato, sans-serif".to_string()),
				font_size: Some(32.0),
				font_weight: Some("700".to_string()),
				font_style: None,
				text_anchor: None,
				content: "RUST".to_string(),
				children: vec![SvgTspan {
					id: None,
					style: SvgStyle::default(),
					x: None,
					y: None,
					dx: None,
					dy: None,
					content: "U".to_string(),
				}],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 1);
		if let FusionTool::SText(txt) = &fusion_doc.tools[0] {
			assert_eq!(txt.name, "brand");
			assert_eq!(txt.styled_text, "RUST");
			assert_eq!(txt.font.as_deref(), Some("Lato"));
			assert_eq!(txt.style.as_deref(), Some("Bold"));
		} else {
			return Err("Expected SText tool".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
