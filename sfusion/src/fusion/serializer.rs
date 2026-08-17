use std::fmt::Write;
use crate::ast::{FusionDoc, FusionTool, PolylinePoint, SMerge, SPolygon, SText, ViewInfo};

// region:    --- Public Functions

/// Serializes a `FusionDoc` into DaVinci Resolve Fusion Lua table format.
pub fn serialize_fusion_doc(doc: &FusionDoc) -> String {
	let mut out = String::new();
	out.push_str("{\n\tTools = ordered() {\n");

	let tool_count = doc.tools.len();
	for (idx, tool) in doc.tools.iter().enumerate() {
		let is_last = idx + 1 == tool_count;
		match tool {
			FusionTool::SPolygon(poly) => serialize_spolygon(&mut out, poly, is_last),
			FusionTool::SMerge(merge) => serialize_smerge(&mut out, merge, is_last),
			FusionTool::SText(text) => serialize_stext(&mut out, text, is_last),
		}
	}

	out.push_str("\t}\n}\n");
	out
}

// endregion: --- Public Functions

// region:    --- Support

fn serialize_spolygon(out: &mut String, poly: &SPolygon, is_last: bool) {
	let trailing_comma = if is_last { "" } else { "," };
	let _ = writeln!(out, "\t\t{} = sPolygon {{", poly.name);
	out.push_str("\t\t\tDrawMode = \"ModifyOnly\",\n");
	out.push_str("\t\t\tNameSet = true,\n");
	out.push_str("\t\t\tInputs = {\n");

	let mask_width = format_f64(poly.mask_width);
	let mask_height = format_f64(poly.mask_height);

	let _ = writeln!(
		out,
		"\t\t\t\tMaskWidth = Input {{\n\t\t\t\t\tValue = Number {{\n\t\t\t\t\t\tValue = {mask_width}\n\t\t\t\t\t}},\n\t\t\t\t}},"
	);
	out.push_str("\t\t\t\tPixelAspect = Input {\n\t\t\t\t\tValue = Point {\n\t\t\t\t\t\tX = 1,\n\t\t\t\t\t\tY = 1\n\t\t\t\t\t},\n\t\t\t\t},\n");
	let _ = writeln!(
		out,
		"\t\t\t\tMaskHeight = Input {{\n\t\t\t\t\tValue = Number {{\n\t\t\t\t\t\tValue = {mask_height}\n\t\t\t\t\t}},\n\t\t\t\t}},"
	);
	if let Some(red) = poly.red {
		let val = format_f64(red);
		let _ = writeln!(out, "\t\t\t\tRed = Input {{ Value = {val}, }},");
	}
	if let Some(green) = poly.green {
		let val = format_f64(green);
		let _ = writeln!(out, "\t\t\t\tGreen = Input {{ Value = {val}, }},");
	}
	if let Some(blue) = poly.blue {
		let val = format_f64(blue);
		let _ = writeln!(out, "\t\t\t\tBlue = Input {{ Value = {val}, }},");
	}
	if let Some(opacity) = poly.opacity {
		let val = format_f64(opacity);
		let _ = writeln!(out, "\t\t\t\tOpacity = Input {{ Value = {val}, }},");
	}
	out.push_str("\t\t\t\tPolyline2 = Input {\n\t\t\t\t\tValue = Polyline {\n\t\t\t\t\t},\n\t\t\t\t},\n");
	if let Some(border_width) = poly.border_width {
		let bw_str = format_f64(border_width);
		let _ = writeln!(
			out,
			"\t\t\t\tBorderWidth = Input {{ Value = {bw_str}, }},"
		);
	}
	out.push_str("\t\t\t\tJoinStyle = Input { Value = 2, },\n");
	out.push_str("\t\t\t\tMiterLimit = Input { Value = 4, },\n");
	out.push_str("\t\t\t\tCapStyle = Input { Value = 0, },\n");
	out.push_str("\t\t\t\tPolyline = Input {\n");
	out.push_str("\t\t\t\t\tValue = Polyline {\n");

	if poly.closed {
		out.push_str("\t\t\t\t\t\tClosed = true,\n");
	}

	out.push_str("\t\t\t\t\t\tPoints = {\n");
	let point_count = poly.points.len();
	for (idx, pt) in poly.points.iter().enumerate() {
		let pt_comma = if idx + 1 == point_count { "" } else { "," };
		serialize_point(out, pt, pt_comma);
	}
	out.push_str("\t\t\t\t\t\t}\n");
	out.push_str("\t\t\t\t\t},\n");
	out.push_str("\t\t\t\t}\n");
	out.push_str("\t\t\t},\n");

	serialize_view_info(out, &poly.view_info, trailing_comma);
}

fn serialize_stext(out: &mut String, text: &SText, is_last: bool) {
	let trailing_comma = if is_last { "" } else { "," };
	let _ = writeln!(out, "\t\t{} = sText {{", text.name);
	out.push_str("\t\t\tNameSet = true,\n");
	out.push_str("\t\t\tInputs = {\n");
	if let Some(wrap) = text.wrap {
		let _ = writeln!(out, "\t\t\t\tWrap = Input {{ Value = {wrap}, }},");
	}
	if let Some(rot) = text.layout_rotation {
		let _ = writeln!(out, "\t\t\t\tLayoutRotation = Input {{ Value = {rot}, }},");
	}
	if let Some(rot) = text.transform_rotation {
		let _ = writeln!(out, "\t\t\t\tTransformRotation = Input {{ Value = {rot}, }},");
	}
	let escaped_text = escape_lua_string(&text.styled_text);
	let _ = writeln!(
		out,
		"\t\t\t\tStyledText = Input {{ Value = \"{escaped_text}\", }},"
	);
	if let Some(font) = &text.font {
		let _ = writeln!(out, "\t\t\t\tFont = Input {{ Value = \"{font}\", }},");
	}
	if let Some(style) = &text.style {
		let _ = writeln!(out, "\t\t\t\tStyle = Input {{ Value = \"{style}\", }},");
	}
	if let Some(ls) = text.line_spacing {
		let val = format_f64(ls);
		let _ = writeln!(out, "\t\t\t\tLineSpacing = Input {{ Value = {val}, }},");
	}
	if let Some(cs) = text.character_spacing {
		let val = format_f64(cs);
		let _ = writeln!(out, "\t\t\t\tCharacterSpacing = Input {{ Value = {val}, }},");
	}
	if let Some(opacity) = text.opacity {
		let val = format_f64(opacity);
		let _ = writeln!(out, "\t\t\t\tOpacity1 = Input {{ Value = {val}, }},");
	}
	if let Some(red) = text.red {
		let val = format_f64(red);
		let _ = writeln!(out, "\t\t\t\tRed1 = Input {{ Value = {val}, }},");
	}
	if let Some(green) = text.green {
		let val = format_f64(green);
		let _ = writeln!(out, "\t\t\t\tGreen1 = Input {{ Value = {val}, }},");
	}
	if let Some(blue) = text.blue {
		let val = format_f64(blue);
		let _ = writeln!(out, "\t\t\t\tBlue1 = Input {{ Value = {val}, }},");
	}
	if let Some(vj) = text.vertical_justification {
		let _ = writeln!(out, "\t\t\t\tVerticalJustificationNew = Input {{ Value = {vj}, }},");
	}
	if let Some(hj) = text.horizontal_justification {
		let _ = writeln!(out, "\t\t\t\tHorizontalJustificationNew = Input {{ Value = {hj}, }},");
	}
	if let Some(hlcr) = text.horizontal_left_center_right {
		let _ = writeln!(out, "\t\t\t\tHorizontalLeftCenterRight = Input {{ Value = {hlcr}, }},");
	}
	if let (Some(cx), Some(cy)) = (text.center_x, text.center_y) {
		let x_str = format_f64(cx);
		let y_str = format_f64(cy);
		let _ = writeln!(
			out,
			"\t\t\t\tCenter = Input {{\n\t\t\t\t\tValue = Point {{\n\t\t\t\t\t\tX = {x_str},\n\t\t\t\t\t\tY = {y_str}\n\t\t\t\t\t}},\n\t\t\t\t}},"
		);
	}
	out.push_str("\t\t\t},\n");
	serialize_view_info(out, &text.view_info, trailing_comma);
}

fn serialize_point(out: &mut String, pt: &PolylinePoint, trailing_comma: &str) {
	let x = format_f64(pt.x);
	let y = format_f64(pt.y);
	let lx = format_f64(pt.lx);
	let ly = format_f64(pt.ly);
	let rx = format_f64(pt.rx);
	let ry = format_f64(pt.ry);

	if pt.linear {
		let _ = writeln!(
			out,
			"\t\t\t\t\t\t\t{{ Linear = true, X = {x}, Y = {y}, LX = {lx}, LY = {ly}, RX = {rx}, RY = {ry} }}{trailing_comma}"
		);
	} else {
		let _ = writeln!(
			out,
			"\t\t\t\t\t\t\t{{ X = {x}, Y = {y}, LX = {lx}, LY = {ly}, RX = {rx}, RY = {ry} }}{trailing_comma}"
		);
	}
}

fn serialize_smerge(out: &mut String, merge: &SMerge, is_last: bool) {
	let trailing_comma = if is_last { "" } else { "," };
	let _ = writeln!(out, "\t\t{} = sMerge {{", merge.name);
	out.push_str("\t\t\tNameSet = true,\n");
	out.push_str("\t\t\tInputs = {\n");

	let input_count = merge.inputs.len();
	for (idx, input_op) in merge.inputs.iter().enumerate() {
		let is_last_input = idx + 1 == input_count;
		let input_idx = idx + 1;
		let comma = if is_last_input { "" } else { "," };
		let _ = writeln!(
			out,
			"\t\t\t\tInput{input_idx} = Input {{\n\t\t\t\t\tSourceOp = \"{input_op}\",\n\t\t\t\t\tSource = \"Output\",\n\t\t\t\t}}{comma}"
		);
	}

	out.push_str("\t\t\t},\n");
	serialize_view_info(out, &merge.view_info, trailing_comma);
}

fn serialize_view_info(out: &mut String, view_info: &ViewInfo, trailing_comma: &str) {
	let pos_x = format_f64(view_info.pos_x);
	let pos_y = format_f64(view_info.pos_y);
	let _ = writeln!(
		out,
		"\t\t\tViewInfo = OperatorInfo {{ Pos = {{ {pos_x}, {pos_y} }} }},\n\t\t}}{trailing_comma}"
	);
}

fn format_f64(val: f64) -> String {
	if val.abs() < 1e-15 {
		"0".to_string()
	} else {
		val.to_string()
	}
}

fn escape_lua_string(val: &str) -> String {
	let mut out = String::with_capacity(val.len());
	for ch in val.chars() {
		match ch {
			'\\' => out.push_str("\\\\"),
			'\"' => out.push_str("\\\""),
			'\n' => out.push_str("\\n"),
			'\r' => out.push_str("\\r"),
			'\t' => out.push_str("\\t"),
			other => out.push(other),
		}
	}
	out
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_fusion_serializer_spolygon_and_merge() -> Result<()> {
		// -- Setup & Fixtures
		let doc = FusionDoc {
			tools: vec![
				FusionTool::SPolygon(SPolygon {
					name: "poly_1".to_string(),
					mask_width: 320.0,
					mask_height: 240.0,
					border_width: Some(0.022),
					red: Some(0.12),
					green: Some(0.34),
					blue: Some(0.56),
					opacity: Some(0.78),
					points: vec![
						PolylinePoint {
							x: 0.1,
							y: 0.2,
							lx: -0.01,
							ly: 0.02,
							rx: 0.01,
							ry: -0.02,
							linear: false,
						},
						PolylinePoint {
							x: 0.3,
							y: 0.4,
							lx: 0.0,
							ly: 0.0,
							rx: 0.0,
							ry: 0.0,
							linear: true,
						},
					],
					closed: true,
					view_info: ViewInfo::new(1980.0, -247.5),
				}),
				FusionTool::SMerge(SMerge {
					name: "loop".to_string(),
					inputs: vec!["poly_1".to_string()],
					view_info: ViewInfo::new(2090.0, -181.5),
				}),
			],
		};

		// -- Exec
		let output = serialize_fusion_doc(&doc);

		// -- Check
		assert!(output.starts_with("{\n\tTools = ordered() {\n"));
		assert!(output.contains("poly_1 = sPolygon {"));
		assert!(output.contains("Value = 320"));
		assert!(output.contains("Value = 240"));
		assert!(output.contains("Red = Input { Value = 0.12, },"));
		assert!(output.contains("Green = Input { Value = 0.34, },"));
		assert!(output.contains("Blue = Input { Value = 0.56, },"));
		assert!(output.contains("Opacity = Input { Value = 0.78, },"));
		assert!(output.contains("BorderWidth = Input { Value = 0.022, },"));
		assert!(output.contains("Closed = true,"));
		assert!(output.contains("{ X = 0.1, Y = 0.2, LX = -0.01, LY = 0.02, RX = 0.01, RY = -0.02 },"));
		assert!(output.contains("{ Linear = true, X = 0.3, Y = 0.4, LX = 0, LY = 0, RX = 0, RY = 0 }"));
		assert!(output.contains("loop = sMerge {"));
		assert!(output.contains("SourceOp = \"poly_1\""));
		assert!(output.ends_with("\t}\n}\n"));

		Ok(())
	}

	#[test]
	fn test_fusion_serializer_stext() -> Result<()> {
		// -- Setup & Fixtures
		let doc = FusionDoc {
			tools: vec![FusionTool::SText(SText {
				name: "sText1".to_string(),
				styled_text: "Hello World".to_string(),
				font: Some("Open Sans".to_string()),
				style: Some("Bold".to_string()),
				red: Some(1.0),
				green: Some(0.5),
				blue: Some(0.0),
				opacity: Some(0.9),
				vertical_justification: Some(3),
				horizontal_justification: Some(3),
				horizontal_left_center_right: Some(1),
				center_x: Some(0.5),
				center_y: Some(0.5),
				view_info: ViewInfo::new(3520.0, -379.5),
				..Default::default()
			})],
		};

		// -- Exec
		let output = serialize_fusion_doc(&doc);

		// -- Check
		assert!(output.contains("sText1 = sText {"));
		assert!(output.contains("StyledText = Input { Value = \"Hello World\", },"));
		assert!(output.contains("Font = Input { Value = \"Open Sans\", },"));
		assert!(output.contains("Style = Input { Value = \"Bold\", },"));
		assert!(output.contains("Red1 = Input { Value = 1, },"));
		assert!(output.contains("Green1 = Input { Value = 0.5, },"));
		assert!(output.contains("Blue1 = Input { Value = 0, },"));
		assert!(output.contains("Opacity1 = Input { Value = 0.9, },"));
		assert!(output.contains("VerticalJustificationNew = Input { Value = 3, },"));
		assert!(output.contains("HorizontalJustificationNew = Input { Value = 3, },"));
		assert!(output.contains("HorizontalLeftCenterRight = Input { Value = 1, },"));
		assert!(output.contains("Pos = { 3520, -379.5 }"));

		Ok(())
	}

	#[test]
	fn test_fusion_serializer_stext_multiline_and_escaping() -> Result<()> {
		// -- Setup & Fixtures
		let doc = FusionDoc {
			tools: vec![FusionTool::SText(SText {
				name: "sTextMultiline".to_string(),
				styled_text: "Line 1\nLine \"2\"\nLine \\3\\\tEnd".to_string(),
				font: Some("Roboto".to_string()),
				style: Some("Regular".to_string()),
				line_spacing: Some(1.25),
				character_spacing: Some(1.05),
				wrap: Some(1),
				layout_rotation: Some(1),
				transform_rotation: Some(1),
				view_info: ViewInfo::new(100.0, 200.0),
				..Default::default()
			})],
		};

		// -- Exec
		let output = serialize_fusion_doc(&doc);

		// -- Check
		assert!(output.contains("sTextMultiline = sText {"));
		assert!(output.contains("StyledText = Input { Value = \"Line 1\\nLine \\\"2\\\"\\nLine \\\\3\\\\\\tEnd\", },"));
		assert!(output.contains("Font = Input { Value = \"Roboto\", },"));
		assert!(output.contains("Style = Input { Value = \"Regular\", },"));
		assert!(output.contains("LineSpacing = Input { Value = 1.25, },"));
		assert!(output.contains("CharacterSpacing = Input { Value = 1.05, },"));
		assert!(output.contains("Wrap = Input { Value = 1, },"));
		assert!(output.contains("LayoutRotation = Input { Value = 1, },"));
		assert!(output.contains("TransformRotation = Input { Value = 1, },"));

		Ok(())
	}
}

// endregion: --- Tests
