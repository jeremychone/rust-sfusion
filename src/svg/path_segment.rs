// region:    --- Types

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
	pub x: f64,
	pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedSegment {
	MoveTo(Point),
	LineTo(Point),
	CubicTo { p1: Point, p2: Point, p: Point },
	Close,
}

// endregion: --- Types

// region:    --- Constructors

impl Point {
	pub fn new(x: f64, y: f64) -> Self {
		Self { x, y }
	}
}

// endregion: --- Constructors

// region:    --- Froms

impl From<(f64, f64)> for Point {
	fn from((x, y): (f64, f64)) -> Self {
		Self { x, y }
	}
}

impl From<Point> for (f64, f64) {
	fn from(pt: Point) -> Self {
		(pt.x, pt.y)
	}
}

// endregion: --- Froms
