// region:    --- Types

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
	pub d: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgRect {
	pub id: Option<String>,
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
	pub cx: f64,
	pub cy: f64,
	pub r: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgEllipse {
	pub id: Option<String>,
	pub cx: f64,
	pub cy: f64,
	pub rx: f64,
	pub ry: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgLine {
	pub id: Option<String>,
	pub x1: f64,
	pub y1: f64,
	pub x2: f64,
	pub y2: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolyline {
	pub id: Option<String>,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolygon {
	pub id: Option<String>,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgGroup {
	pub id: Option<String>,
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
