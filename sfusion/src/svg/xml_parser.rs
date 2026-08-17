use std::collections::HashMap;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::ast::*;
use crate::error::{Error, Result};
use crate::svg::style_parser::{
	parse_color, parse_dimension, parse_fill_rule, parse_paint, parse_stroke_dasharray, parse_stroke_linecap,
	parse_stroke_linejoin, parse_style_str,
};

// region:    --- Types

enum CurrentGradient {
	Linear(SvgLinearGradient),
	Radial(SvgRadialGradient),
}

// endregion: --- Types

// region:    --- Public Functions

/// Parses an SVG XML string into an `SvgDoc`.
pub fn parse_svg(xml: &str) -> Result<SvgDoc> {
	let mut reader = Reader::from_str(xml);
	reader.config_mut().trim_text(true);

	let mut doc = SvgDoc {
		view_box: None,
		width: None,
		height: None,
		defs: SvgDefs::default(),
		elements: Vec::new(),
	};

	let mut group_stack: Vec<SvgGroup> = Vec::new();
	let mut current_gradient: Option<CurrentGradient> = None;
	let mut current_text: Option<SvgText> = None;
	let mut current_tspan: Option<SvgTspan> = None;
	let mut buf = Vec::new();

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(ref e)) => {
				let local_name = e.local_name();
				match local_name.as_ref() {
					b"svg" => {
						parse_svg_attributes(e, &mut doc)?;
					}
					b"defs" => {
						// Defs container, children parsed directly
					}
					b"linearGradient" => {
						current_gradient = Some(CurrentGradient::Linear(parse_linear_gradient(e)?));
					}
					b"radialGradient" => {
						current_gradient = Some(CurrentGradient::Radial(parse_radial_gradient(e)?));
					}
					b"stop" => {
						if let Some(ref mut grad) = current_gradient {
							let stop = parse_gradient_stop(e)?;
							match grad {
								CurrentGradient::Linear(l) => l.stops.push(stop),
								CurrentGradient::Radial(r) => r.stops.push(stop),
							}
						}
					}
					b"text" => {
						current_text = Some(parse_text_start(e)?);
					}
					b"tspan" => {
						current_tspan = Some(parse_tspan_start(e)?);
					}
					b"g" => {
						let id = get_attribute_str(e, b"id")?;
						let transform = get_attribute_transform(e)?;
						let style = parse_element_style(e)?;
						group_stack.push(SvgGroup {
							id,
							transform,
							style,
							children: Vec::new(),
						});
					}
					_ => {
						if let Some(element) = parse_element(e)? {
							append_element(&mut doc, &mut group_stack, element);
						}
					}
				}
			}
			Ok(Event::Empty(ref e)) => {
				let local_name = e.local_name();
				match local_name.as_ref() {
					b"svg" => {
						parse_svg_attributes(e, &mut doc)?;
					}
					b"linearGradient" => {
						let grad = parse_linear_gradient(e)?;
						doc.defs.gradients.insert(grad.id.clone(), SvgGradient::Linear(grad));
					}
					b"radialGradient" => {
						let grad = parse_radial_gradient(e)?;
						doc.defs.gradients.insert(grad.id.clone(), SvgGradient::Radial(grad));
					}
					b"stop" => {
						if let Some(ref mut grad) = current_gradient {
							let stop = parse_gradient_stop(e)?;
							match grad {
								CurrentGradient::Linear(l) => l.stops.push(stop),
								CurrentGradient::Radial(r) => r.stops.push(stop),
							}
						}
					}
					b"text" => {
						let text = parse_text_start(e)?;
						append_element(&mut doc, &mut group_stack, SvgElement::Text(text));
					}
					b"tspan" => {
						let tspan = parse_tspan_start(e)?;
						if let Some(ref mut text) = current_text {
							text.children.push(tspan);
						}
					}
					_ => {
						if let Some(element) = parse_element(e)? {
							append_element(&mut doc, &mut group_stack, element);
						}
					}
				}
			}
			Ok(Event::End(ref e)) => {
				let local_name = e.local_name();
				match local_name.as_ref() {
					b"linearGradient" | b"radialGradient" => {
						if let Some(grad) = current_gradient.take() {
							match grad {
								CurrentGradient::Linear(l) => {
									doc.defs.gradients.insert(l.id.clone(), SvgGradient::Linear(l));
								}
								CurrentGradient::Radial(r) => {
									doc.defs.gradients.insert(r.id.clone(), SvgGradient::Radial(r));
								}
							}
						}
					}
					b"tspan" => {
						if let Some(tspan) = current_tspan.take()
							&& let Some(ref mut text) = current_text
						{
							text.children.push(tspan);
						}
					}
					b"text" => {
						if let Some(text) = current_text.take() {
							append_element(&mut doc, &mut group_stack, SvgElement::Text(text));
						}
					}
					b"g" => {
						if let Some(group) = group_stack.pop() {
							let group_elem = SvgElement::Group(group);
							append_element(&mut doc, &mut group_stack, group_elem);
						}
					}
					_ => {}
				}
			}
			Ok(Event::Text(ref e)) => {
				let text_val = e
					.unescape()
					.map_err(|err| Error::custom(format!("Invalid text content: {err}")))?;
				if let Some(ref mut tspan) = current_tspan {
					tspan.content.push_str(&text_val);
				}
				if let Some(ref mut text) = current_text {
					text.content.push_str(&text_val);
				}
			}
			Ok(Event::CData(ref e)) => {
				let text_val = String::from_utf8_lossy(e.as_ref());
				if let Some(ref mut tspan) = current_tspan {
					tspan.content.push_str(&text_val);
				}
				if let Some(ref mut text) = current_text {
					text.content.push_str(&text_val);
				}
			}
			Ok(Event::Eof) => break,
			Err(e) => return Err(Error::custom(format!("XML parse error at position {}: {e}", reader.buffer_position()))),
			_ => {}
		}
		buf.clear();
	}

	Ok(doc)
}

// endregion: --- Public Functions

// region:    --- Support

fn append_element(doc: &mut SvgDoc, group_stack: &mut [SvgGroup], element: SvgElement) {
	if let Some(current_group) = group_stack.last_mut() {
		current_group.children.push(element);
	} else {
		doc.elements.push(element);
	}
}

fn parse_svg_attributes(e: &quick_xml::events::BytesStart<'_>, doc: &mut SvgDoc) -> Result<()> {
	if let Some(vb_str) = get_attribute_str(e, b"viewBox")? {
		doc.view_box = parse_view_box(&vb_str);
	}

	if let Some(w_str) = get_attribute_str(e, b"width")? {
		doc.width = parse_dimension(&w_str);
	}

	if let Some(h_str) = get_attribute_str(e, b"height")? {
		doc.height = parse_dimension(&h_str);
	}

	Ok(())
}

fn parse_linear_gradient(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgLinearGradient> {
	let id = get_attribute_str(e, b"id")?.unwrap_or_default();
	let x1 = get_attribute_f64(e, b"x1")?;
	let y1 = get_attribute_f64(e, b"y1")?;
	let x2 = get_attribute_f64(e, b"x2")?;
	let y2 = get_attribute_f64(e, b"y2")?;
	let transform = get_attribute_transform(e)?.or_else(|| get_attribute_gradient_transform(e).ok().flatten());

	Ok(SvgLinearGradient {
		id,
		x1,
		y1,
		x2,
		y2,
		stops: Vec::new(),
		transform,
	})
}

fn parse_radial_gradient(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgRadialGradient> {
	let id = get_attribute_str(e, b"id")?.unwrap_or_default();
	let cx = get_attribute_f64(e, b"cx")?;
	let cy = get_attribute_f64(e, b"cy")?;
	let r = get_attribute_f64(e, b"r")?;
	let fx = get_attribute_f64(e, b"fx")?;
	let fy = get_attribute_f64(e, b"fy")?;
	let transform = get_attribute_transform(e)?.or_else(|| get_attribute_gradient_transform(e).ok().flatten());

	Ok(SvgRadialGradient {
		id,
		cx,
		cy,
		r,
		fx,
		fy,
		stops: Vec::new(),
		transform,
	})
}

fn parse_text_start(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgText> {
	let id = get_attribute_str(e, b"id")?;
	let transform = get_attribute_transform(e)?;
	let style = parse_element_style(e)?;
	let x = get_attribute_f64(e, b"x")?;
	let y = get_attribute_f64(e, b"y")?;
	let dx = get_attribute_f64(e, b"dx")?;
	let dy = get_attribute_f64(e, b"dy")?;

	let mut font_family = get_attribute_str(e, b"font-family")?
		.or_else(|| get_attribute_str(e, b"font_family").ok().flatten());
	let mut font_size = get_attribute_f64(e, b"font-size")?
		.or_else(|| get_attribute_f64(e, b"font_size").ok().flatten());
	let mut font_weight = get_attribute_str(e, b"font-weight")?
		.or_else(|| get_attribute_str(e, b"font_weight").ok().flatten());
	let mut font_style = get_attribute_str(e, b"font-style")?
		.or_else(|| get_attribute_str(e, b"font_style").ok().flatten());
	let mut text_anchor = get_attribute_str(e, b"text-anchor")?
		.or_else(|| get_attribute_str(e, b"text_anchor").ok().flatten());

	if let Some(ref extra) = style.extra {
		if font_family.is_none() {
			font_family = extra.get("font-family").cloned().or_else(|| extra.get("font_family").cloned());
		}
		if font_size.is_none() {
			font_size = extra
				.get("font-size")
				.or_else(|| extra.get("font_size"))
				.and_then(|v| parse_dimension(v));
		}
		if font_weight.is_none() {
			font_weight = extra.get("font-weight").cloned().or_else(|| extra.get("font_weight").cloned());
		}
		if font_style.is_none() {
			font_style = extra.get("font-style").cloned().or_else(|| extra.get("font_style").cloned());
		}
		if text_anchor.is_none() {
			text_anchor = extra.get("text-anchor").cloned().or_else(|| extra.get("text_anchor").cloned());
		}
	}

	Ok(SvgText {
		id,
		transform,
		style,
		x,
		y,
		dx,
		dy,
		font_family,
		font_size,
		font_weight,
		font_style,
		text_anchor,
		content: String::new(),
		children: Vec::new(),
	})
}

fn parse_tspan_start(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgTspan> {
	let id = get_attribute_str(e, b"id")?;
	let style = parse_element_style(e)?;
	let x = get_attribute_f64(e, b"x")?;
	let y = get_attribute_f64(e, b"y")?;
	let dx = get_attribute_f64(e, b"dx")?;
	let dy = get_attribute_f64(e, b"dy")?;

	Ok(SvgTspan {
		id,
		style,
		x,
		y,
		dx,
		dy,
		content: String::new(),
	})
}

fn parse_gradient_stop(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgGradientStop> {
	let offset = get_attribute_f64(e, b"offset")?.unwrap_or(0.0);

	let mut color = None;
	let mut opacity = None;

	if let Some(color_str) = get_attribute_str(e, b"stop-color")? {
		color = parse_color(&color_str);
	}
	if let Some(op_str) = get_attribute_str(e, b"stop-opacity")? {
		opacity = parse_dimension(&op_str);
	}

	if let Some(style_str) = get_attribute_str(e, b"style")? {
		for decl in style_str.split(';') {
			let mut parts = decl.splitn(2, ':');
			if let Some(k) = parts.next()
				&& let Some(v) = parts.next()
			{
				match k.trim() {
					"stop-color" => {
						if let Some(c) = parse_color(v.trim()) {
							color = Some(c);
						}
					}
					"stop-opacity" => {
						if let Some(op) = parse_dimension(v.trim()) {
							opacity = Some(op);
						}
					}
					_ => {}
				}
			}
		}
	}

	Ok(SvgGradientStop {
		offset,
		color: color.unwrap_or_else(|| SvgColor::new_rgb(0, 0, 0)),
		opacity,
	})
}

fn parse_element(e: &quick_xml::events::BytesStart<'_>) -> Result<Option<SvgElement>> {
	let local_name = e.local_name();
	let id = get_attribute_str(e, b"id")?;
	let transform = get_attribute_transform(e)?;
	let style = parse_element_style(e)?;

	match local_name.as_ref() {
		b"path" => {
			let d = get_attribute_str(e, b"d")?.unwrap_or_default();
			Ok(Some(SvgElement::Path(SvgPath { id, transform, style, d })))
		}
		b"rect" => {
			let x = get_attribute_f64(e, b"x")?.unwrap_or(0.0);
			let y = get_attribute_f64(e, b"y")?.unwrap_or(0.0);
			let width = get_attribute_f64(e, b"width")?.unwrap_or(0.0);
			let height = get_attribute_f64(e, b"height")?.unwrap_or(0.0);
			let rx = get_attribute_f64(e, b"rx")?;
			let ry = get_attribute_f64(e, b"ry")?;
			Ok(Some(SvgElement::Rect(SvgRect {
				id,
				transform,
				style,
				x,
				y,
				width,
				height,
				rx,
				ry,
			})))
		}
		b"circle" => {
			let cx = get_attribute_f64(e, b"cx")?.unwrap_or(0.0);
			let cy = get_attribute_f64(e, b"cy")?.unwrap_or(0.0);
			let r = get_attribute_f64(e, b"r")?.unwrap_or(0.0);
			Ok(Some(SvgElement::Circle(SvgCircle { id, transform, style, cx, cy, r })))
		}
		b"ellipse" => {
			let cx = get_attribute_f64(e, b"cx")?.unwrap_or(0.0);
			let cy = get_attribute_f64(e, b"cy")?.unwrap_or(0.0);
			let rx = get_attribute_f64(e, b"rx")?.unwrap_or(0.0);
			let ry = get_attribute_f64(e, b"ry")?.unwrap_or(0.0);
			Ok(Some(SvgElement::Ellipse(SvgEllipse { id, transform, style, cx, cy, rx, ry })))
		}
		b"line" => {
			let x1 = get_attribute_f64(e, b"x1")?.unwrap_or(0.0);
			let y1 = get_attribute_f64(e, b"y1")?.unwrap_or(0.0);
			let x2 = get_attribute_f64(e, b"x2")?.unwrap_or(0.0);
			let y2 = get_attribute_f64(e, b"y2")?.unwrap_or(0.0);
			Ok(Some(SvgElement::Line(SvgLine { id, transform, style, x1, y1, x2, y2 })))
		}
		b"polyline" => {
			let points_str = get_attribute_str(e, b"points")?.unwrap_or_default();
			let points = parse_points_list(&points_str);
			Ok(Some(SvgElement::Polyline(SvgPolyline { id, transform, style, points })))
		}
		b"polygon" => {
			let points_str = get_attribute_str(e, b"points")?.unwrap_or_default();
			let points = parse_points_list(&points_str);
			Ok(Some(SvgElement::Polygon(SvgPolygon { id, transform, style, points })))
		}
		_ => Ok(None),
	}
}

fn get_attribute_str(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Result<Option<String>> {
	for attr in e.attributes().flatten() {
		if attr.key.as_ref() == key {
			let val = attr
				.unescape_value()
				.map_err(|err| Error::custom(format!("Invalid attribute value: {err}")))?;
			return Ok(Some(val.into_owned()));
		}
	}
	Ok(None)
}

fn get_attribute_f64(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Result<Option<f64>> {
	if let Some(val_str) = get_attribute_str(e, key)?
		&& let Some(num) = parse_dimension(&val_str)
	{
		return Ok(Some(num));
	}
	Ok(None)
}

fn get_attribute_transform(e: &quick_xml::events::BytesStart<'_>) -> Result<Option<Transform2D>> {
	if let Some(t_str) = get_attribute_str(e, b"transform")? {
		Ok(parse_transform(&t_str))
	} else {
		Ok(None)
	}
}

fn get_attribute_gradient_transform(e: &quick_xml::events::BytesStart<'_>) -> Result<Option<Transform2D>> {
	if let Some(t_str) = get_attribute_str(e, b"gradientTransform")? {
		Ok(parse_transform(&t_str))
	} else {
		Ok(None)
	}
}

fn parse_element_style(e: &quick_xml::events::BytesStart<'_>) -> Result<SvgStyle> {
	let mut pres_style = SvgStyle::default();
	let mut inline_style: Option<SvgStyle> = None;

	for attr in e.attributes().flatten() {
		let key = attr.key.as_ref();
		let val = attr
			.unescape_value()
			.map_err(|err| Error::custom(format!("Invalid attribute value: {err}")))?;
		let val_str = val.trim();
		if val_str.is_empty() {
			continue;
		}

		match key {
			b"fill" => pres_style.fill = parse_paint(val_str),
			b"fill-opacity" => pres_style.fill_opacity = parse_dimension(val_str),
			b"fill-rule" => pres_style.fill_rule = parse_fill_rule(val_str),
			b"stroke" => pres_style.stroke = parse_paint(val_str),
			b"stroke-width" => pres_style.stroke_width = parse_dimension(val_str),
			b"stroke-opacity" => pres_style.stroke_opacity = parse_dimension(val_str),
			b"stroke-linecap" => pres_style.stroke_linecap = parse_stroke_linecap(val_str),
			b"stroke-linejoin" => pres_style.stroke_linejoin = parse_stroke_linejoin(val_str),
			b"stroke-miterlimit" => pres_style.stroke_miterlimit = parse_dimension(val_str),
			b"stroke-dasharray" => pres_style.stroke_dasharray = parse_stroke_dasharray(val_str),
			b"stroke-dashoffset" => pres_style.stroke_dashoffset = parse_dimension(val_str),
			b"opacity" => pres_style.opacity = parse_dimension(val_str),
			b"color" => {
				pres_style
					.extra
					.get_or_insert_with(HashMap::new)
					.entry("color".to_string())
					.or_insert_with(|| val_str.to_string());
			}
			b"style" => {
				inline_style = Some(parse_style_str(val_str));
			}
			_ => {}
		}
	}

	if let Some(inline) = inline_style {
		Ok(inline.inherit_from(&pres_style))
	} else {
		Ok(pres_style)
	}
}

fn parse_view_box(s: &str) -> Option<SvgViewBox> {
	let nums: Vec<f64> = s
		.split(|c: char| c.is_whitespace() || c == ',')
		.filter(|item| !item.is_empty())
		.filter_map(|item| item.parse::<f64>().ok())
		.collect();

	if nums.len() == 4 {
		Some(SvgViewBox::new(nums[0], nums[1], nums[2], nums[3]))
	} else {
		None
	}
}

fn parse_points_list(s: &str) -> Vec<(f64, f64)> {
	let nums: Vec<f64> = s
		.split(|c: char| c.is_whitespace() || c == ',')
		.filter(|item| !item.is_empty())
		.filter_map(|item| item.parse::<f64>().ok())
		.collect();

	nums.chunks_exact(2).map(|chunk| (chunk[0], chunk[1])).collect()
}

pub fn parse_transform(s: &str) -> Option<Transform2D> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	let mut current = Transform2D::identity();
	let mut has_transform = false;
	let mut rest = s;

	while let Some(open_paren) = rest.find('(') {
		let func_name = rest[..open_paren].trim().trim_start_matches(|c: char| c == ',' || c.is_whitespace());
		let after_open = &rest[open_paren + 1..];
		let close_paren = match after_open.find(')') {
			Some(idx) => idx,
			None => break,
		};
		let args_str = &after_open[..close_paren];
		rest = &after_open[close_paren + 1..];

		let nums: Vec<f64> = args_str
			.split(|c: char| c.is_whitespace() || c == ',')
			.filter(|item| !item.is_empty())
			.filter_map(|item| item.parse::<f64>().ok())
			.collect();

		let tf = match func_name {
			"matrix" if nums.len() >= 6 => Some(Transform2D {
				a: nums[0],
				b: nums[1],
				c: nums[2],
				d: nums[3],
				e: nums[4],
				f: nums[5],
			}),
			"translate" if !nums.is_empty() => {
				let tx = nums[0];
				let ty = nums.get(1).copied().unwrap_or(0.0);
				Some(Transform2D {
					a: 1.0,
					b: 0.0,
					c: 0.0,
					d: 1.0,
					e: tx,
					f: ty,
				})
			}
			"scale" if !nums.is_empty() => {
				let sx = nums[0];
				let sy = nums.get(1).copied().unwrap_or(sx);
				Some(Transform2D {
					a: sx,
					b: 0.0,
					c: 0.0,
					d: sy,
					e: 0.0,
					f: 0.0,
				})
			}
			"rotate" if !nums.is_empty() => {
				let rad = nums[0].to_radians();
				let cos = rad.cos();
				let sin = rad.sin();
				if nums.len() >= 3 {
					let cx = nums[1];
					let cy = nums[2];
					let t1 = Transform2D { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: cx, f: cy };
					let r = Transform2D { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 };
					let t2 = Transform2D { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: -cx, f: -cy };
					Some(t1.multiply(&r).multiply(&t2))
				} else {
					Some(Transform2D { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 })
				}
			}
			_ => None,
		};

		if let Some(matrix) = tf {
			current = current.multiply(&matrix);
			has_transform = true;
		}
	}

	if has_transform {
		Some(current)
	} else {
		None
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_svg_parser_simple_doc() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 320 240" width="320" height="240">
			<path id="poly_1" d="M 10 20 L 30 40 Z"/>
			<rect id="grabber" x="10" y="20" width="50" height="60"/>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.view_box, Some(SvgViewBox::new(0.0, 0.0, 320.0, 240.0)));
		assert_eq!(doc.width, Some(320.0));
		assert_eq!(doc.height, Some(240.0));
		assert_eq!(doc.elements.len(), 2);

		match &doc.elements[0] {
			SvgElement::Path(p) => {
				assert_eq!(p.id.as_deref(), Some("poly_1"));
				assert_eq!(p.d, "M 10 20 L 30 40 Z");
			}
			_ => return Err("Expected Path element".into()),
		}

		match &doc.elements[1] {
			SvgElement::Rect(r) => {
				assert_eq!(r.id.as_deref(), Some("grabber"));
				assert_eq!(r.width, 50.0);
				assert_eq!(r.height, 60.0);
			}
			_ => return Err("Expected Rect element".into()),
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_stroke_width() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 100 100">
			<g id="g1" stroke-width="2.5">
				<path id="p1" d="M 0 0 L 10 10" stroke-width="1.0"/>
				<rect id="r1" x="0" y="0" width="10" height="10" style="stroke-width: 3.5px; fill: none"/>
			</g>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		if let SvgElement::Group(g) = &doc.elements[0] {
			assert_eq!(g.style.stroke_width, Some(2.5));
			assert_eq!(g.children.len(), 2);

			if let SvgElement::Path(p) = &g.children[0] {
				assert_eq!(p.style.stroke_width, Some(1.0));
			} else {
				return Err("Expected path child".into());
			}

			if let SvgElement::Rect(r) = &g.children[1] {
				assert_eq!(r.style.stroke_width, Some(3.5));
			} else {
				return Err("Expected rect child".into());
			}
		} else {
			return Err("Expected group element".into());
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_defs_and_gradients() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r##"<svg viewBox="0 0 100 100">
			<defs>
				<linearGradient id="grad_linear" x1="0%" y1="0%" x2="100%" y2="100%" gradientTransform="rotate(45)">
					<stop offset="0%" stop-color="#ff0000" stop-opacity="1"/>
					<stop offset="100%" style="stop-color: #0000ff; stop-opacity: 0.5"/>
				</linearGradient>
				<radialGradient id="grad_radial" cx="0.5" cy="0.5" r="0.5" fx="0.2" fy="0.2">
					<stop offset="0" stop-color="white"/>
					<stop offset="1" stop-color="black"/>
				</radialGradient>
			</defs>
			<rect id="r1" x="0" y="0" width="100" height="100"/>
		</svg>"##;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.defs.gradients.len(), 2);

		let linear = match doc.defs.gradients.get("grad_linear") {
			Some(SvgGradient::Linear(l)) => l,
			_ => return Err("Expected linear gradient grad_linear".into()),
		};
		assert_eq!(linear.id, "grad_linear");
		assert_eq!(linear.x1, Some(0.0));
		assert_eq!(linear.y1, Some(0.0));
		assert_eq!(linear.x2, Some(1.0));
		assert_eq!(linear.y2, Some(1.0));
		assert!(linear.transform.is_some());
		assert_eq!(linear.stops.len(), 2);
		assert_eq!(linear.stops[0].offset, 0.0);
		assert_eq!(linear.stops[0].color, SvgColor::new_rgb(255, 0, 0));
		assert_eq!(linear.stops[0].opacity, Some(1.0));
		assert_eq!(linear.stops[1].offset, 1.0);
		assert_eq!(linear.stops[1].color, SvgColor::new_rgb(0, 0, 255));
		assert_eq!(linear.stops[1].opacity, Some(0.5));

		let radial = match doc.defs.gradients.get("grad_radial") {
			Some(SvgGradient::Radial(r)) => r,
			_ => return Err("Expected radial gradient grad_radial".into()),
		};
		assert_eq!(radial.id, "grad_radial");
		assert_eq!(radial.cx, Some(0.5));
		assert_eq!(radial.cy, Some(0.5));
		assert_eq!(radial.r, Some(0.5));
		assert_eq!(radial.fx, Some(0.2));
		assert_eq!(radial.fy, Some(0.2));
		assert_eq!(radial.stops.len(), 2);
		assert_eq!(radial.stops[0].color, SvgColor::new_rgb(255, 255, 255));
		assert_eq!(radial.stops[1].color, SvgColor::new_rgb(0, 0, 0));

		Ok(())
	}

	#[test]
	fn test_svg_parser_element_styles_and_inline_override() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r##"<svg viewBox="0 0 100 100">
			<rect id="r1" fill="red" stroke="#00ff00" stroke-width="4" fill-opacity="0.5" stroke-linecap="round" stroke-linejoin="bevel" stroke-dasharray="2, 4" opacity="0.9"/>
			<circle id="c1" cx="50" cy="50" r="20" fill="red" stroke="blue" stroke-width="2" style="fill: yellow; stroke-width: 6px"/>
		</svg>"##;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 2);

		if let SvgElement::Rect(r) = &doc.elements[0] {
			assert_eq!(r.style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(255, 0, 0))));
			assert_eq!(r.style.stroke, Some(SvgPaint::Color(SvgColor::new_rgb(0, 255, 0))));
			assert_eq!(r.style.stroke_width, Some(4.0));
			assert_eq!(r.style.fill_opacity, Some(0.5));
			assert_eq!(r.style.stroke_linecap, Some(StrokeLinecap::Round));
			assert_eq!(r.style.stroke_linejoin, Some(StrokeLinejoin::Bevel));
			assert_eq!(r.style.stroke_dasharray, Some(vec![2.0, 4.0]));
			assert_eq!(r.style.opacity, Some(0.9));
		} else {
			return Err("Expected Rect element".into());
		}

		if let SvgElement::Circle(c) = &doc.elements[1] {
			assert_eq!(c.style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(255, 255, 0))));
			assert_eq!(c.style.stroke, Some(SvgPaint::Color(SvgColor::new_rgb(0, 0, 255))));
			assert_eq!(c.style.stroke_width, Some(6.0));
		} else {
			return Err("Expected Circle element".into());
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_nested_groups() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 100 100">
			<g id="group_1">
				<circle id="c1" cx="10" cy="10" r="5"/>
				<g id="inner_group">
					<line id="l1" x1="0" y1="0" x2="10" y2="10"/>
				</g>
			</g>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		if let SvgElement::Group(g) = &doc.elements[0] {
			assert_eq!(g.id.as_deref(), Some("group_1"));
			assert_eq!(g.children.len(), 2);
			if let SvgElement::Group(inner) = &g.children[1] {
				assert_eq!(inner.id.as_deref(), Some("inner_group"));
				assert_eq!(inner.children.len(), 1);
			} else {
				return Err("Expected inner group".into());
			}
		} else {
			return Err("Expected top group".into());
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_text_element_simple() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r##"<svg viewBox="0 0 400 200">
			<text id="title_txt" x="20" y="50" font-family="Roboto" font-size="28" font-weight="bold" font-style="italic" text-anchor="middle" fill="#ff5500">
				Hello Fusion
			</text>
		</svg>"##;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		match &doc.elements[0] {
			SvgElement::Text(t) => {
				assert_eq!(t.id.as_deref(), Some("title_txt"));
				assert_eq!(t.x, Some(20.0));
				assert_eq!(t.y, Some(50.0));
				assert_eq!(t.font_family.as_deref(), Some("Roboto"));
				assert_eq!(t.font_size, Some(28.0));
				assert_eq!(t.font_weight.as_deref(), Some("bold"));
				assert_eq!(t.font_style.as_deref(), Some("italic"));
				assert_eq!(t.text_anchor.as_deref(), Some("middle"));
				assert_eq!(t.content, "Hello Fusion");
				assert_eq!(t.style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(255, 85, 0))));
			}
			_ => return Err("Expected Text element".into()),
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_text_with_inline_style() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 300 100">
			<text id="styled_text" x="10" y="30" style="font-family: 'Open Sans'; font-size: 18px; font-weight: 600; text-anchor: end; fill: #112233">
				Styled Text
			</text>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		if let SvgElement::Text(t) = &doc.elements[0] {
			assert_eq!(t.id.as_deref(), Some("styled_text"));
			assert_eq!(t.font_family.as_deref(), Some("'Open Sans'"));
			assert_eq!(t.font_size, Some(18.0));
			assert_eq!(t.font_weight.as_deref(), Some("600"));
			assert_eq!(t.text_anchor.as_deref(), Some("end"));
			assert_eq!(t.content, "Styled Text");
		} else {
			return Err("Expected Text element".into());
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_text_with_tspans() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 500 200">
			<text id="multiline" x="10" y="40">
				Base text
				<tspan id="span_1" x="10" y="70" fill="red">First line</tspan>
				<tspan id="span_2" x="10" y="100" fill="blue">Second line</tspan>
			</text>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		if let SvgElement::Text(t) = &doc.elements[0] {
			assert_eq!(t.id.as_deref(), Some("multiline"));
			assert_eq!(t.content, "Base textFirst lineSecond line");
			assert_eq!(t.children.len(), 2);
			assert_eq!(t.children[0].id.as_deref(), Some("span_1"));
			assert_eq!(t.children[0].x, Some(10.0));
			assert_eq!(t.children[0].y, Some(70.0));
			assert_eq!(t.children[0].content, "First line");
			assert_eq!(t.children[0].style.fill, Some(SvgPaint::Color(SvgColor::new_rgb(255, 0, 0))));

			assert_eq!(t.children[1].id.as_deref(), Some("span_2"));
			assert_eq!(t.children[1].content, "Second line");
		} else {
			return Err("Expected Text element".into());
		}

		Ok(())
	}

	#[test]
	fn test_svg_parser_nested_tspan_flattening() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 500 200">
			<text id="brand" x="50" y="100">
				R<tspan fill="red">U</tspan>ST
			</text>
		</svg>"#;

		// -- Exec
		let doc = parse_svg(xml)?;

		// -- Check
		assert_eq!(doc.elements.len(), 1);
		if let SvgElement::Text(t) = &doc.elements[0] {
			assert_eq!(t.id.as_deref(), Some("brand"));
			assert_eq!(t.content, "RUST");
			assert_eq!(t.children.len(), 1);
			assert_eq!(t.children[0].content, "U");
		} else {
			return Err("Expected Text element".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
