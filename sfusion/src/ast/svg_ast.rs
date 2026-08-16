// region:    --- Types

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
	pub a: f64,
	pub b: f64,
	pub c: f64,
	pub d: f64,
	pub e: f64,
	pub f: f64,
}

impl Default for Transform2D {
	fn default() -> Self {
		Self::identity()
	}
}

impl Transform2D {
	pub fn identity() -> Self {
		Self {
			a: 1.0,
			b: 0.0,
			c: 0.0,
			d: 1.0,
			e: 0.0,
			f: 0.0,
		}
	}

	pub fn multiply(&self, other: &Transform2D) -> Transform2D {
		Transform2D {
			a: self.a * other.a + self.c * other.b,
			b: self.b * other.a + self.d * other.b,
			c: self.a * other.c + self.c * other.d,
			d: self.b * other.c + self.d * other.d,
			e: self.a * other.e + self.c * other.f + self.e,
			f: self.b * other.e + self.d * other.f + self.f,
		}
	}

	pub fn transform_xy(&self, x: f64, y: f64) -> (f64, f64) {
		(
			self.a * x + self.c * y + self.e,
			self.b * x + self.d * y + self.f,
		)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SvgDoc {
	pub view_box: Option<SvgViewBox>,
	pub width: Option<f64>,
	pub height: Option<f64>,
	pub elements: Vec<SvgElement>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SvgViewBox {
	pub min_x: f64,
	pub min_y: f64,
	pub width: f64,
	pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgElement {
	Path(SvgPath),
	Rect(SvgRect),
	Circle(SvgCircle),
	Ellipse(SvgEllipse),
	Line(SvgLine),
	Polyline(SvgPolyline),
	Polygon(SvgPolygon),
	Group(SvgGroup),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPath {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub d: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgRect {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub x: f64,
	pub y: f64,
	pub width: f64,
	pub height: f64,
	pub rx: Option<f64>,
	pub ry: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgCircle {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub cx: f64,
	pub cy: f64,
	pub r: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgEllipse {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub cx: f64,
	pub cy: f64,
	pub rx: f64,
	pub ry: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgLine {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub x1: f64,
	pub y1: f64,
	pub x2: f64,
	pub y2: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolyline {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolygon {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgGroup {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub stroke_width: Option<f64>,
	pub children: Vec<SvgElement>,
}

// endregion: --- Types

// region:    --- Constructors

impl SvgDoc {
	pub fn effective_view_box(&self) -> SvgViewBox {
		if let Some(vb) = self.view_box {
			vb
		} else {
			SvgViewBox {
				min_x: 0.0,
				min_y: 0.0,
				width: self.width.unwrap_or(100.0),
				height: self.height.unwrap_or(100.0),
			}
		}
	}
}

impl SvgViewBox {
	pub fn new(min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
		Self {
			min_x,
			min_y,
			width,
			height,
		}
	}
}

// endregion: --- Constructors
