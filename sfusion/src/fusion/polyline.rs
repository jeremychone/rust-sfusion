// region:    --- Types

use crate::ast::{PolylinePoint, SvgElement, SvgViewBox};
use crate::error::Result;
use crate::svg::{element_to_segments, NormalizedSegment, Point};

pub const TARGET_ASPECT_RATIO: f64 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PolylineData {
	pub points: Vec<PolylinePoint>,
	pub closed: bool,
}

// endregion: --- Types

// region:    --- Public Functions

/// Transform an SVG point into Fusion centered coordinates with inverted Y and aspect ratio compensation.
pub fn transform_point_with_aspect(pt: Point, view_box: &SvgViewBox, target_aspect: f64) -> Point {
	let width = if view_box.width == 0.0 { 1.0 } else { view_box.width };
	let height = if view_box.height == 0.0 { 1.0 } else { view_box.height };
	let aspect = if target_aspect == 0.0 { 1.0 } else { target_aspect };

	let center_x = view_box.min_x + width / 2.0;
	let center_y = view_box.min_y + height / 2.0;

	let rel_x = pt.x - center_x;
	let rel_y = pt.y - center_y;

	let nx = rel_x / (height * aspect);
	let ny = -rel_y / height;

	Point::new(nx, ny)
}

/// Transform an SVG point into Fusion centered coordinates with inverted Y using default 1:1 target aspect ratio.
pub fn transform_point(pt: Point, view_box: &SvgViewBox) -> Point {
	transform_point_with_aspect(pt, view_box, TARGET_ASPECT_RATIO)
}

/// Transform an SVG vector delta into Fusion normalized delta with aspect ratio compensation.
pub fn transform_vector_with_aspect(dx: f64, dy: f64, view_box: &SvgViewBox, target_aspect: f64) -> (f64, f64) {
	let height = if view_box.height == 0.0 { 1.0 } else { view_box.height };
	let aspect = if target_aspect == 0.0 { 1.0 } else { target_aspect };

	let ndx = dx / (height * aspect);
	let ndy = -dy / height;

	(ndx, ndy)
}

/// Transform an SVG vector delta into Fusion normalized delta using default 1:1 target aspect ratio.
pub fn transform_vector(dx: f64, dy: f64, view_box: &SvgViewBox) -> (f64, f64) {
	transform_vector_with_aspect(dx, dy, view_box, TARGET_ASPECT_RATIO)
}

/// Converts normalized SVG path segments into one or more Fusion polylines with computed tangent handles using custom aspect ratio.
pub fn segments_to_polylines_with_aspect(
	segments: &[NormalizedSegment],
	view_box: &SvgViewBox,
	target_aspect: f64,
) -> Vec<PolylineData> {
	if segments.is_empty() {
		return Vec::new();
	}

	let mut polylines = Vec::new();
	let mut current_points: Vec<PolylinePoint> = Vec::new();
	let mut current_closed = false;

	for segment in segments {
		match segment {
			NormalizedSegment::MoveTo(pt) => {
				if !current_points.is_empty() {
					polylines.push(PolylineData {
						points: std::mem::take(&mut current_points),
						closed: current_closed,
					});
					current_closed = false;
				}

				let f_pt = transform_point_with_aspect(*pt, view_box, target_aspect);
				current_points.push(PolylinePoint {
					x: f_pt.x,
					y: f_pt.y,
					lx: 0.0,
					ly: 0.0,
					rx: 0.0,
					ry: 0.0,
					linear: true,
				});
			}

			NormalizedSegment::LineTo(pt) => {
				let f_pt = transform_point_with_aspect(*pt, view_box, target_aspect);
				current_points.push(PolylinePoint {
					x: f_pt.x,
					y: f_pt.y,
					lx: 0.0,
					ly: 0.0,
					rx: 0.0,
					ry: 0.0,
					linear: true,
				});
			}

			NormalizedSegment::CubicTo { p1, p2, p } => {
				// Outgoing tangent for previous point
				let f_p1 = transform_point_with_aspect(*p1, view_box, target_aspect);
				let f_p2 = transform_point_with_aspect(*p2, view_box, target_aspect);
				let f_p = transform_point_with_aspect(*p, view_box, target_aspect);

				if let Some(prev) = current_points.last_mut() {
					prev.rx = f_p1.x - prev.x;
					prev.ry = f_p1.y - prev.y;
					prev.linear = false;
				}

				current_points.push(PolylinePoint {
					x: f_p.x,
					y: f_p.y,
					lx: f_p2.x - f_p.x,
					ly: f_p2.y - f_p.y,
					rx: 0.0,
					ry: 0.0,
					linear: false,
				});
			}

			NormalizedSegment::Close => {
				current_closed = true;
				if current_points.len() > 1 {
					let first = current_points[0];
					let last = current_points[current_points.len() - 1];

					// Check if last point is duplicate of first point
					if (first.x - last.x).abs() < 1e-6 && (first.y - last.y).abs() < 1e-6 {
						current_points[0].lx = last.lx;
						current_points[0].ly = last.ly;
						current_points.pop();
					}
				}
			}
		}
	}

	if !current_points.is_empty() {
		polylines.push(PolylineData {
			points: current_points,
			closed: current_closed,
		});
	}

	polylines
}

/// Converts normalized SVG path segments into one or more Fusion polylines with computed tangent handles.
pub fn segments_to_polylines(segments: &[NormalizedSegment], view_box: &SvgViewBox) -> Vec<PolylineData> {
	segments_to_polylines_with_aspect(segments, view_box, TARGET_ASPECT_RATIO)
}

/// Converts any SVG element directly into Fusion polylines using custom aspect ratio.
pub fn element_to_polylines_with_aspect(
	element: &SvgElement,
	view_box: &SvgViewBox,
	target_aspect: f64,
) -> Result<Vec<PolylineData>> {
	let segments = element_to_segments(element)?;
	Ok(segments_to_polylines_with_aspect(&segments, view_box, target_aspect))
}

/// Converts any SVG element directly into Fusion polylines.
pub fn element_to_polylines(element: &SvgElement, view_box: &SvgViewBox) -> Result<Vec<PolylineData>> {
	element_to_polylines_with_aspect(element, view_box, TARGET_ASPECT_RATIO)
}

// endregion: --- Public Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::ast::SvgRect;

	#[test]
	fn test_fusion_polyline_coord_transform() -> Result<()> {
		// -- Setup & Fixtures
		let vb = SvgViewBox::new(0.0, 0.0, 100.0, 100.0);

		// -- Exec
		let center = transform_point(Point::new(50.0, 50.0), &vb);
		let top_left = transform_point(Point::new(0.0, 0.0), &vb);
		let bottom_right = transform_point(Point::new(100.0, 100.0), &vb);

		// -- Check
		assert!((center.x - 0.0).abs() < 1e-6);
		assert!((center.y - 0.0).abs() < 1e-6);

		assert!((top_left.x - (-0.5)).abs() < 1e-6);
		assert!((top_left.y - 0.5).abs() < 1e-6);

		assert!((bottom_right.x - 0.5).abs() < 1e-6);
		assert!((bottom_right.y - (-0.5)).abs() < 1e-6);

		Ok(())
	}

	#[test]
	fn test_fusion_polyline_rect_conversion() -> Result<()> {
		// -- Setup & Fixtures
		let rect = SvgElement::Rect(SvgRect {
			id: None,
			transform: None,
			style: crate::ast::SvgStyle::default(),
			x: 0.0,
			y: 0.0,
			width: 100.0,
			height: 100.0,
			rx: None,
			ry: None,
		});
		let vb = SvgViewBox::new(0.0, 0.0, 100.0, 100.0);

		// -- Exec
		let polylines = element_to_polylines(&rect, &vb)?;

		// -- Check
		assert_eq!(polylines.len(), 1);
		assert!(polylines[0].closed);
		assert_eq!(polylines[0].points.len(), 4);

		for pt in &polylines[0].points {
			assert!(pt.linear);
			assert_eq!(pt.lx, 0.0);
			assert_eq!(pt.ly, 0.0);
			assert_eq!(pt.rx, 0.0);
			assert_eq!(pt.ry, 0.0);
		}

		Ok(())
	}

	#[test]
	fn test_fusion_polyline_cubic_tangents() -> Result<()> {
		// -- Setup & Fixtures
		let vb = SvgViewBox::new(0.0, 0.0, 100.0, 100.0);
		let segments = vec![
			NormalizedSegment::MoveTo(Point::new(0.0, 50.0)),
			NormalizedSegment::CubicTo {
				p1: Point::new(25.0, 0.0),
				p2: Point::new(75.0, 0.0),
				p: Point::new(100.0, 50.0),
			},
		];

		// -- Exec
		let polylines = segments_to_polylines(&segments, &vb);

		// -- Check
		assert_eq!(polylines.len(), 1);
		assert_eq!(polylines[0].points.len(), 2);

		let p0 = polylines[0].points[0];
		let p1 = polylines[0].points[1];

		assert!(!p0.linear);
		assert!((p0.rx - 0.25).abs() < 1e-6);
		assert!((p0.ry - 0.5).abs() < 1e-6);

		assert!(!p1.linear);
		assert!((p1.lx - (-0.25)).abs() < 1e-6);
		assert!((p1.ly - 0.5).abs() < 1e-6);

		Ok(())
	}

	#[test]
	fn test_fusion_polyline_aspect_ratio_tangents() -> Result<()> {
		// -- Setup & Fixtures
		let vb = SvgViewBox::new(0.0, 0.0, 100.0, 100.0);
		let segments = vec![
			NormalizedSegment::MoveTo(Point::new(0.0, 50.0)),
			NormalizedSegment::CubicTo {
				p1: Point::new(25.0, 0.0),
				p2: Point::new(75.0, 0.0),
				p: Point::new(100.0, 50.0),
			},
		];

		// -- Exec
		let polylines_16_9 = segments_to_polylines_with_aspect(&segments, &vb, 16.0 / 9.0);
		let polylines_1_1 = segments_to_polylines_with_aspect(&segments, &vb, 1.0);

		// -- Check
		let p0_16_9 = polylines_16_9[0].points[0];
		let p0_1_1 = polylines_1_1[0].points[0];

		// X tangent is scaled by 1/aspect
		assert!((p0_16_9.rx - (p0_1_1.rx / (16.0 / 9.0))).abs() < 1e-6);
		// Y tangent remains identical regardless of horizontal aspect ratio
		assert!((p0_16_9.ry - p0_1_1.ry).abs() < 1e-6);

		let p1_16_9 = polylines_16_9[0].points[1];
		let p1_1_1 = polylines_1_1[0].points[1];
		assert!((p1_16_9.lx - (p1_1_1.lx / (16.0 / 9.0))).abs() < 1e-6);
		assert!((p1_16_9.ly - p1_1_1.ly).abs() < 1e-6);
		Ok(())
	}

	#[test]
	fn test_fusion_polyline_arc_tangents() -> Result<()> {
		// -- Setup & Fixtures
		let vb = SvgViewBox::new(0.0, 0.0, 100.0, 100.0);
		let circle = SvgElement::Circle(crate::ast::SvgCircle {
			id: None,
			transform: None,
			style: crate::ast::SvgStyle::default(),
			cx: 50.0,
			cy: 50.0,
			r: 50.0,
		});

		// -- Exec
		let polylines = element_to_polylines(&circle, &vb)?;

		// -- Check
		assert_eq!(polylines.len(), 1);
		assert!(polylines[0].closed);
		assert_eq!(polylines[0].points.len(), 4);

		for pt in &polylines[0].points {
			assert!(!pt.linear);
			// All handle lengths should be uniform under 1:1 isotropic scaling
			let handle_len_l = (pt.lx * pt.lx + pt.ly * pt.ly).sqrt();
			let handle_len_r = (pt.rx * pt.rx + pt.ry * pt.ry).sqrt();
			assert!(handle_len_l > 0.2 && handle_len_l < 0.3);
			assert!(handle_len_r > 0.2 && handle_len_r < 0.3);
		}

		Ok(())
	}
}

// endregion: --- Tests
