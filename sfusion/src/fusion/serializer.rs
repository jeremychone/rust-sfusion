use std::fmt::Write;
use crate::ast::{FusionDoc, FusionTool, PolylinePoint, SMerge, SPolygon, ViewInfo};

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
		assert!(output.contains("BorderWidth = Input { Value = 0.022, },"));
		assert!(output.contains("Closed = true,"));
		assert!(output.contains("{ X = 0.1, Y = 0.2, LX = -0.01, LY = 0.02, RX = 0.01, RY = -0.02 },"));
		assert!(output.contains("{ Linear = true, X = 0.3, Y = 0.4, LX = 0, LY = 0, RX = 0, RY = 0 }"));
		assert!(output.contains("loop = sMerge {"));
		assert!(output.contains("SourceOp = \"poly_1\""));
		assert!(output.ends_with("\t}\n}\n"));

		Ok(())
	}
}

// endregion: --- Tests
