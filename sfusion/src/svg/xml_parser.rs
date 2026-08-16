use quick_xml::events::Event;
use quick_xml::Reader;

use crate::ast::*;
use crate::error::{Error, Result};

// region:    --- Public Functions

/// Parses an SVG XML string into an `SvgDoc`.
pub fn parse_svg(xml: &str) -> Result<SvgDoc> {
	let mut reader = Reader::from_str(xml);
	reader.config_mut().trim_text(true);

	let mut doc = SvgDoc {
		view_box: None,
		width: None,
		height: None,
		elements: Vec::new(),
	};

	let mut group_stack: Vec<SvgGroup> = Vec::new();
	let mut buf = Vec::new();

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(ref e)) => {
				let local_name = e.local_name();
				match local_name.as_ref() {
					b"svg" => {
						parse_svg_attributes(e, &mut doc)?;
					}
					b"g" => {
						let id = get_attribute_str(e, b"id")?;
						let transform = get_attribute_transform(e)?;
						let stroke_width = get_attribute_stroke_width(e)?;
						group_stack.push(SvgGroup {
							id,
							transform,
							stroke_width,
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
					_ => {
						if let Some(element) = parse_element(e)? {
							append_element(&mut doc, &mut group_stack, element);
						}
					}
				}
			}
			Ok(Event::End(ref e)) => {
				let local_name = e.local_name();
				if local_name.as_ref() == b"g"
					&& let Some(group) = group_stack.pop()
				{
					let group_elem = SvgElement::Group(group);
					append_element(&mut doc, &mut group_stack, group_elem);
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

fn parse_element(e: &quick_xml::events::BytesStart<'_>) -> Result<Option<SvgElement>> {
	let local_name = e.local_name();
	let id = get_attribute_str(e, b"id")?;
	let transform = get_attribute_transform(e)?;
	let stroke_width = get_attribute_stroke_width(e)?;

	match local_name.as_ref() {
		b"path" => {
			let d = get_attribute_str(e, b"d")?.unwrap_or_default();
			Ok(Some(SvgElement::Path(SvgPath { id, transform, stroke_width, d })))
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
				stroke_width,
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
			Ok(Some(SvgElement::Circle(SvgCircle { id, transform, stroke_width, cx, cy, r })))
		}
		b"ellipse" => {
			let cx = get_attribute_f64(e, b"cx")?.unwrap_or(0.0);
			let cy = get_attribute_f64(e, b"cy")?.unwrap_or(0.0);
			let rx = get_attribute_f64(e, b"rx")?.unwrap_or(0.0);
			let ry = get_attribute_f64(e, b"ry")?.unwrap_or(0.0);
			Ok(Some(SvgElement::Ellipse(SvgEllipse { id, transform, stroke_width, cx, cy, rx, ry })))
		}
		b"line" => {
			let x1 = get_attribute_f64(e, b"x1")?.unwrap_or(0.0);
			let y1 = get_attribute_f64(e, b"y1")?.unwrap_or(0.0);
			let x2 = get_attribute_f64(e, b"x2")?.unwrap_or(0.0);
			let y2 = get_attribute_f64(e, b"y2")?.unwrap_or(0.0);
			Ok(Some(SvgElement::Line(SvgLine { id, transform, stroke_width, x1, y1, x2, y2 })))
		}
		b"polyline" => {
			let points_str = get_attribute_str(e, b"points")?.unwrap_or_default();
			let points = parse_points_list(&points_str);
			Ok(Some(SvgElement::Polyline(SvgPolyline { id, transform, stroke_width, points })))
		}
		b"polygon" => {
			let points_str = get_attribute_str(e, b"points")?.unwrap_or_default();
			let points = parse_points_list(&points_str);
			Ok(Some(SvgElement::Polygon(SvgPolygon { id, transform, stroke_width, points })))
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

fn get_attribute_stroke_width(e: &quick_xml::events::BytesStart<'_>) -> Result<Option<f64>> {
	if let Some(val) = get_attribute_f64(e, b"stroke-width")? {
		return Ok(Some(val));
	}

	if let Some(style_str) = get_attribute_str(e, b"style")?
		&& let Some(val) = parse_style_stroke_width(&style_str)
	{
		return Ok(Some(val));
	}

	Ok(None)
}

fn parse_style_stroke_width(style: &str) -> Option<f64> {
	for decl in style.split(';') {
		let mut parts = decl.splitn(2, ':');
		if let Some(key) = parts.next()
			&& let Some(val) = parts.next()
			&& key.trim() == "stroke-width"
		{
			return parse_dimension(val);
		}
	}
	None
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

fn parse_dimension(s: &str) -> Option<f64> {
	let s = s.trim();
	let s = s.trim_end_matches("px").trim_end_matches("pt").trim();
	s.parse::<f64>().ok()
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
			assert_eq!(g.stroke_width, Some(2.5));
			assert_eq!(g.children.len(), 2);

			if let SvgElement::Path(p) = &g.children[0] {
				assert_eq!(p.stroke_width, Some(1.0));
			} else {
				return Err("Expected path child".into());
			}

			if let SvgElement::Rect(r) = &g.children[1] {
				assert_eq!(r.stroke_width, Some(3.5));
			} else {
				return Err("Expected rect child".into());
			}
		} else {
			return Err("Expected group element".into());
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
}

// endregion: --- Tests
