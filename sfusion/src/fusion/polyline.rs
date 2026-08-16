// region:    --- Types

use crate::ast::{PolylinePoint, SvgElement, SvgViewBox};
use crate::error::Result;
use crate::svg::{element_to_segments, NormalizedSegment, Point};

#[derive(Debug, Clone, PartialEq)]
pub struct PolylineData {
	pub points: Vec<PolylinePoint>,
	pub closed: bool,
}

// endregion: --- Types

// region:    --- Public Functions

/// Transform an SVG point into Fusion centered coordinates with inverted Y.
pub fn transform_point(pt: Point, view_box: &SvgViewBox) -> Point {
	let width = if view_box.width == 0.0 { 1.0 } else { view_box.width };
	let height = if view_box.height == 0.0 { 1.0 } else { view_box.height };

	let nx = (pt.x - view_box.min_x) / width - 0.5;
	let ny = 0.5 - (pt.y - view_box.min_y) / height;

	Point::new(nx, ny)
}

/// Transform an SVG vector delta into Fusion normalized delta.
pub fn transform_vector(dx: f64, dy: f64, view_box: &SvgViewBox) -> (f64, f64) {
	let width = if view_box.width == 0.0 { 1.0 } else { view_box.width };
	let height = if view_box.height == 0.0 { 1.0 } else { view_box.height };

	let ndx = dx / width;
	let ndy = -dy / height;

	(ndx, ndy)
}

/// Converts normalized SVG path segments into one or more Fusion polylines with computed tangent handles.
pub fn segments_to_polylines(segments: &[NormalizedSegment], view_box: &SvgViewBox) -> Vec<PolylineData> {
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

				let f_pt = transform_point(*pt, view_box);
				current_points.push(PolylinePoint {
					x: f_pt.x,
					y: f_pt.y,
					lx: 0.0,
					ly: 0.0,
					rx: 0.0,
					ry: 0.0,
					linear: false,
				});
			}

			NormalizedSegment::LineTo(pt) => {
				let f_pt = transform_point(*pt, view_box);
				if let Some(prev) = current_points.last_mut() {
					prev.rx = (f_pt.x - prev.x) / 3.0;
					prev.ry = (f_pt.y - prev.y) / 3.0;
				}

				let prev_pt = current_points.last().copied().unwrap_or(PolylinePoint::new_linear(0.0, 0.0));
				current_points.push(PolylinePoint {
					x: f_pt.x,
					y: f_pt.y,
					lx: (prev_pt.x - f_pt.x) / 3.0,
					ly: (prev_pt.y - f_pt.y) / 3.0,
					rx: 0.0,
					ry: 0.0,
					linear: true,
				});
			}

			NormalizedSegment::CubicTo { p1, p2, p } => {
				// Outgoing tangent for previous point
				let f_p1 = transform_point(*p1, view_box);
				let f_p2 = transform_point(*p2, view_box);
				let f_p = transform_point(*p, view_box);

				if let Some(prev) = current_points.last_mut() {
					prev.rx = f_p1.x - prev.x;
					prev.ry = f_p1.y - prev.y;
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
					} else {
						let last_idx = current_points.len() - 1;
						current_points[last_idx].rx = (first.x - last.x) / 3.0;
						current_points[last_idx].ry = (first.y - last.y) / 3.0;
						current_points[0].lx = (last.x - first.x) / 3.0;
						current_points[0].ly = (last.y - first.y) / 3.0;
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

/// Converts any SVG element directly into Fusion polylines.
pub fn element_to_polylines(element: &SvgElement, view_box: &SvgViewBox) -> Result<Vec<PolylineData>> {
	let segments = element_to_segments(element)?;
	Ok(segments_to_polylines(&segments, view_box))
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

		for pt in &polylines[0].points[1..] {
			assert!(pt.linear);
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
}

// endregion: --- Tests
