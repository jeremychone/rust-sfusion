use super::path_parser::parse_svg_path;
use super::path_segment::{NormalizedSegment, Point};
use crate::ast::{SvgCircle, SvgElement, SvgEllipse, SvgLine, SvgPath, SvgPolygon, SvgPolyline, SvgRect};
use crate::error::Result;

// Constant for cubic bezier circular arc approximation: 4 * (sqrt(2) - 1) / 3
const KAPPA: f64 = 0.552_284_749_830_793_5;

// region:    --- Public Functions

/// Converts any standard SVG element into a list of normalized path segments.
pub fn element_to_segments(element: &SvgElement) -> Result<Vec<NormalizedSegment>> {
	match element {
		SvgElement::Path(path) => path_to_segments(path),
		SvgElement::Rect(rect) => Ok(rect_to_segments(rect)),
		SvgElement::Circle(circle) => Ok(circle_to_segments(circle)),
		SvgElement::Ellipse(ellipse) => Ok(ellipse_to_segments(ellipse)),
		SvgElement::Line(line) => Ok(line_to_segments(line)),
		SvgElement::Polyline(polyline) => Ok(polyline_to_segments(polyline)),
		SvgElement::Polygon(polygon) => Ok(polygon_to_segments(polygon)),
		SvgElement::Group(_) => Ok(Vec::new()),
	}
}

pub fn path_to_segments(path: &SvgPath) -> Result<Vec<NormalizedSegment>> {
	parse_svg_path(&path.d)
}

pub fn rect_to_segments(rect: &SvgRect) -> Vec<NormalizedSegment> {
	let x = rect.x;
	let y = rect.y;
	let w = rect.width;
	let h = rect.height;

	let (rx, ry) = match (rect.rx, rect.ry) {
		(Some(rx), Some(ry)) => (rx, ry),
		(Some(rx), None) => (rx, rx),
		(None, Some(ry)) => (ry, ry),
		(None, None) => (0.0, 0.0),
	};

	let rx = rx.clamp(0.0, w / 2.0);
	let ry = ry.clamp(0.0, h / 2.0);

	if rx <= 0.0 || ry <= 0.0 {
		vec![
			NormalizedSegment::MoveTo(Point::new(x, y)),
			NormalizedSegment::LineTo(Point::new(x + w, y)),
			NormalizedSegment::LineTo(Point::new(x + w, y + h)),
			NormalizedSegment::LineTo(Point::new(x, y + h)),
			NormalizedSegment::Close,
		]
	} else {
		vec![
			NormalizedSegment::MoveTo(Point::new(x + rx, y)),
			NormalizedSegment::LineTo(Point::new(x + w - rx, y)),
			NormalizedSegment::CubicTo {
				p1: Point::new(x + w - rx + KAPPA * rx, y),
				p2: Point::new(x + w, y + ry - KAPPA * ry),
				p: Point::new(x + w, y + ry),
			},
			NormalizedSegment::LineTo(Point::new(x + w, y + h - ry)),
			NormalizedSegment::CubicTo {
				p1: Point::new(x + w, y + h - ry + KAPPA * ry),
				p2: Point::new(x + w - rx + KAPPA * rx, y + h),
				p: Point::new(x + w - rx, y + h),
			},
			NormalizedSegment::LineTo(Point::new(x + rx, y + h)),
			NormalizedSegment::CubicTo {
				p1: Point::new(x + rx - KAPPA * rx, y + h),
				p2: Point::new(x, y + h - ry + KAPPA * ry),
				p: Point::new(x, y + h - ry),
			},
			NormalizedSegment::LineTo(Point::new(x, y + ry)),
			NormalizedSegment::CubicTo {
				p1: Point::new(x, y + ry - KAPPA * ry),
				p2: Point::new(x + rx - KAPPA * rx, y),
				p: Point::new(x + rx, y),
			},
			NormalizedSegment::Close,
		]
	}
}

pub fn circle_to_segments(circle: &SvgCircle) -> Vec<NormalizedSegment> {
	ellipse_to_segments(&SvgEllipse {
		id: circle.id.clone(),
		transform: circle.transform,
		stroke_width: circle.stroke_width,
		cx: circle.cx,
		cy: circle.cy,
		rx: circle.r,
		ry: circle.r,
	})
}

pub fn ellipse_to_segments(ellipse: &SvgEllipse) -> Vec<NormalizedSegment> {
	let cx = ellipse.cx;
	let cy = ellipse.cy;
	let rx = ellipse.rx;
	let ry = ellipse.ry;

	vec![
		NormalizedSegment::MoveTo(Point::new(cx + rx, cy)),
		NormalizedSegment::CubicTo {
			p1: Point::new(cx + rx, cy + KAPPA * ry),
			p2: Point::new(cx + KAPPA * rx, cy + ry),
			p: Point::new(cx, cy + ry),
		},
		NormalizedSegment::CubicTo {
			p1: Point::new(cx - KAPPA * rx, cy + ry),
			p2: Point::new(cx - rx, cy + KAPPA * ry),
			p: Point::new(cx - rx, cy),
		},
		NormalizedSegment::CubicTo {
			p1: Point::new(cx - rx, cy - KAPPA * ry),
			p2: Point::new(cx - KAPPA * rx, cy - ry),
			p: Point::new(cx, cy - ry),
		},
		NormalizedSegment::CubicTo {
			p1: Point::new(cx + KAPPA * rx, cy - ry),
			p2: Point::new(cx + rx, cy - KAPPA * ry),
			p: Point::new(cx + rx, cy),
		},
		NormalizedSegment::Close,
	]
}

pub fn line_to_segments(line: &SvgLine) -> Vec<NormalizedSegment> {
	vec![
		NormalizedSegment::MoveTo(Point::new(line.x1, line.y1)),
		NormalizedSegment::LineTo(Point::new(line.x2, line.y2)),
	]
}

pub fn polyline_to_segments(polyline: &SvgPolyline) -> Vec<NormalizedSegment> {
	if polyline.points.is_empty() {
		return Vec::new();
	}

	let mut segments = Vec::with_capacity(polyline.points.len());
	segments.push(NormalizedSegment::MoveTo(Point::new(
		polyline.points[0].0,
		polyline.points[0].1,
	)));

	for pt in &polyline.points[1..] {
		segments.push(NormalizedSegment::LineTo(Point::new(pt.0, pt.1)));
	}

	segments
}

pub fn polygon_to_segments(polygon: &SvgPolygon) -> Vec<NormalizedSegment> {
	if polygon.points.is_empty() {
		return Vec::new();
	}

	let mut segments = Vec::with_capacity(polygon.points.len() + 1);
	segments.push(NormalizedSegment::MoveTo(Point::new(
		polygon.points[0].0,
		polygon.points[0].1,
	)));

	for pt in &polygon.points[1..] {
		segments.push(NormalizedSegment::LineTo(Point::new(pt.0, pt.1)));
	}

	segments.push(NormalizedSegment::Close);
	segments
}

// endregion: --- Public Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_svg_shape_converter_rect_simple() -> Result<()> {
		// -- Setup & Fixtures
		let rect = SvgRect {
			id: None,
			transform: None,
			stroke_width: None,
			x: 10.0,
			y: 20.0,
			width: 100.0,
			height: 50.0,
			rx: None,
			ry: None,
		};

		// -- Exec
		let segments = rect_to_segments(&rect);

		// -- Check
		assert_eq!(segments.len(), 5);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, 20.0)));
		assert_eq!(segments[1], NormalizedSegment::LineTo(Point::new(110.0, 20.0)));
		assert_eq!(segments[2], NormalizedSegment::LineTo(Point::new(110.0, 70.0)));
		assert_eq!(segments[3], NormalizedSegment::LineTo(Point::new(10.0, 70.0)));
		assert_eq!(segments[4], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_shape_converter_rect_rounded() -> Result<()> {
		// -- Setup & Fixtures
		let rect = SvgRect {
			id: None,
			transform: None,
			stroke_width: None,
			x: 0.0,
			y: 0.0,
			width: 100.0,
			height: 80.0,
			rx: Some(10.0),
			ry: None,
		};

		// -- Exec
		let segments = rect_to_segments(&rect);

		// -- Check
		// 1 MoveTo + 4 LineTo + 4 CubicTo + 1 Close = 10 segments
		assert_eq!(segments.len(), 10);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, 0.0)));
		assert_eq!(segments[1], NormalizedSegment::LineTo(Point::new(90.0, 0.0)));
		assert_eq!(segments[9], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_shape_converter_circle_and_ellipse() -> Result<()> {
		// -- Setup & Fixtures
		let circle = SvgCircle {
			id: None,
			transform: None,
			stroke_width: None,
			cx: 50.0,
			cy: 50.0,
			r: 25.0,
		};

		// -- Exec
		let segments = circle_to_segments(&circle);

		// -- Check
		assert_eq!(segments.len(), 6);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(75.0, 50.0)));
		assert_eq!(segments[5], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_shape_converter_line_polyline_polygon() -> Result<()> {
		// -- Setup & Fixtures
		let line = SvgLine {
			id: None,
			transform: None,
			stroke_width: None,
			x1: 10.0,
			y1: 20.0,
			x2: 30.0,
			y2: 40.0,
		};
		let polyline = SvgPolyline {
			id: None,
			transform: None,
			stroke_width: None,
			points: vec![(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)],
		};
		let polygon = SvgPolygon {
			id: None,
			transform: None,
			stroke_width: None,
			points: vec![(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)],
		};

		// -- Exec
		let line_segs = line_to_segments(&line);
		let polyline_segs = polyline_to_segments(&polyline);
		let polygon_segs = polygon_to_segments(&polygon);

		// -- Check
		assert_eq!(line_segs.len(), 2);
		assert_eq!(polyline_segs.len(), 3);
		assert_eq!(polygon_segs.len(), 4);
		assert_eq!(polygon_segs[3], NormalizedSegment::Close);

		Ok(())
	}
}

// endregion: --- Tests
