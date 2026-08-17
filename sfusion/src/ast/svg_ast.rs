use std::collections::HashMap;

// region:    --- Types

#[derive(Debug, Clone, PartialEq)]
pub enum SvgPaint {
	None,
	CurrentColor,
	Color(SvgColor),
	Url(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SvgColor {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
	NonZero,
	EvenOdd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeLinecap {
	Butt,
	Round,
	Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeLinejoin {
	Miter,
	Round,
	Bevel,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgStyle {
	pub fill: Option<SvgPaint>,
	pub fill_opacity: Option<f64>,
	pub fill_rule: Option<FillRule>,
	pub stroke: Option<SvgPaint>,
	pub stroke_width: Option<f64>,
	pub stroke_opacity: Option<f64>,
	pub stroke_linecap: Option<StrokeLinecap>,
	pub stroke_linejoin: Option<StrokeLinejoin>,
	pub stroke_miterlimit: Option<f64>,
	pub stroke_dasharray: Option<Vec<f64>>,
	pub stroke_dashoffset: Option<f64>,
	pub opacity: Option<f64>,
	pub extra: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SvgGradientStop {
	pub offset: f64,
	pub color: SvgColor,
	pub opacity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgGradient {
	Linear(SvgLinearGradient),
	Radial(SvgRadialGradient),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgLinearGradient {
	pub id: String,
	pub x1: Option<f64>,
	pub y1: Option<f64>,
	pub x2: Option<f64>,
	pub y2: Option<f64>,
	pub stops: Vec<SvgGradientStop>,
	pub transform: Option<Transform2D>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgRadialGradient {
	pub id: String,
	pub cx: Option<f64>,
	pub cy: Option<f64>,
	pub r: Option<f64>,
	pub fx: Option<f64>,
	pub fy: Option<f64>,
	pub stops: Vec<SvgGradientStop>,
	pub transform: Option<Transform2D>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgDefs {
	pub gradients: HashMap<String, SvgGradient>,
}

pub const MAX_1080P_DIMENSION: f64 = 1080.0;

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

	pub fn translate(tx: f64, ty: f64) -> Self {
		Self {
			a: 1.0,
			b: 0.0,
			c: 0.0,
			d: 1.0,
			e: tx,
			f: ty,
		}
	}

	pub fn scale(sx: f64, sy: f64) -> Self {
		Self {
			a: sx,
			b: 0.0,
			c: 0.0,
			d: sy,
			e: 0.0,
			f: 0.0,
		}
	}

	pub fn rotate_rad(rad: f64) -> Self {
		let cos = rad.cos();
		let sin = rad.sin();
		Self {
			a: cos,
			b: sin,
			c: -sin,
			d: cos,
			e: 0.0,
			f: 0.0,
		}
	}

	pub fn rotate_deg(deg: f64) -> Self {
		Self::rotate_rad(deg.to_radians())
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgDoc {
	pub view_box: Option<SvgViewBox>,
	pub width: Option<f64>,
	pub height: Option<f64>,
	pub defs: SvgDefs,
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
	Text(SvgText),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPath {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub d: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgRect {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
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
	pub style: SvgStyle,
	pub cx: f64,
	pub cy: f64,
	pub r: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgEllipse {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub cx: f64,
	pub cy: f64,
	pub rx: f64,
	pub ry: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgLine {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub x1: f64,
	pub y1: f64,
	pub x2: f64,
	pub y2: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolyline {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgPolygon {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgGroup {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub children: Vec<SvgElement>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgText {
	pub id: Option<String>,
	pub transform: Option<Transform2D>,
	pub style: SvgStyle,
	pub x: Option<f64>,
	pub y: Option<f64>,
	pub dx: Option<f64>,
	pub dy: Option<f64>,
	pub font_family: Option<String>,
	pub font_size: Option<f64>,
	pub font_weight: Option<String>,
	pub font_style: Option<String>,
	pub text_anchor: Option<String>,
	pub content: String,
	pub children: Vec<SvgTspan>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgTspan {
	pub id: Option<String>,
	pub style: SvgStyle,
	pub x: Option<f64>,
	pub y: Option<f64>,
	pub dx: Option<f64>,
	pub dy: Option<f64>,
	pub content: String,
}

// endregion: --- Types

// region:    --- Constructors

impl SvgColor {
	pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: 1.0 }
	}

	pub fn new_rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
		Self { r, g, b, a }
	}
}

impl SvgStyle {
	pub fn inherit_from(&self, parent: &SvgStyle) -> SvgStyle {
		SvgStyle {
			fill: self.fill.clone().or_else(|| parent.fill.clone()),
			fill_opacity: self.fill_opacity.or(parent.fill_opacity),
			fill_rule: self.fill_rule.or(parent.fill_rule),
			stroke: self.stroke.clone().or_else(|| parent.stroke.clone()),
			stroke_width: self.stroke_width.or(parent.stroke_width),
			stroke_opacity: self.stroke_opacity.or(parent.stroke_opacity),
			stroke_linecap: self.stroke_linecap.or(parent.stroke_linecap),
			stroke_linejoin: self.stroke_linejoin.or(parent.stroke_linejoin),
			stroke_miterlimit: self.stroke_miterlimit.or(parent.stroke_miterlimit),
			stroke_dasharray: self.stroke_dasharray.clone().or_else(|| parent.stroke_dasharray.clone()),
			stroke_dashoffset: self.stroke_dashoffset.or(parent.stroke_dashoffset),
			opacity: self.opacity.or(parent.opacity),
			extra: match (&self.extra, &parent.extra) {
				(Some(child_map), Some(parent_map)) => {
					let mut merged = parent_map.clone();
					merged.extend(child_map.clone());
					Some(merged)
				}
				(Some(child_map), None) => Some(child_map.clone()),
				(None, Some(parent_map)) => Some(parent_map.clone()),
				(None, None) => None,
			},
		}
	}
}

impl SvgElement {
	pub fn style(&self) -> &SvgStyle {
		match self {
			SvgElement::Path(p) => &p.style,
			SvgElement::Rect(r) => &r.style,
			SvgElement::Circle(c) => &c.style,
			SvgElement::Ellipse(e) => &e.style,
			SvgElement::Line(l) => &l.style,
			SvgElement::Polyline(pl) => &pl.style,
			SvgElement::Polygon(pg) => &pg.style,
			SvgElement::Group(g) => &g.style,
			SvgElement::Text(t) => &t.style,
		}
	}

	pub fn style_mut(&mut self) -> &mut SvgStyle {
		match self {
			SvgElement::Path(p) => &mut p.style,
			SvgElement::Rect(r) => &mut r.style,
			SvgElement::Circle(c) => &mut c.style,
			SvgElement::Ellipse(e) => &mut e.style,
			SvgElement::Line(l) => &mut l.style,
			SvgElement::Polyline(pl) => &mut pl.style,
			SvgElement::Polygon(pg) => &mut pg.style,
			SvgElement::Group(g) => &mut g.style,
			SvgElement::Text(t) => &mut t.style,
		}
	}
}

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

	pub fn center(&self) -> (f64, f64) {
		(self.min_x + self.width / 2.0, self.min_y + self.height / 2.0)
	}

	pub fn scaled_dimensions_to_max(&self, max_dimension: f64) -> (f64, f64) {
		let max_dim = self.width.max(self.height);
		if max_dim <= 0.0 {
			(max_dimension, max_dimension)
		} else {
			let scale = max_dimension / max_dim;
			(self.width * scale, self.height * scale)
		}
	}

	pub fn scaled_1080p_dimensions(&self) -> (f64, f64) {
		self.scaled_dimensions_to_max(MAX_1080P_DIMENSION)
	}
}

// endregion: --- Constructors

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_ast_svg_style_inheritance() -> Result<()> {
		// -- Setup & Fixtures
		let parent = SvgStyle {
			stroke_width: Some(4.0),
			fill: Some(SvgPaint::Color(SvgColor::new_rgb(255, 0, 0))),
			fill_opacity: Some(0.8),
			stroke_linecap: Some(StrokeLinecap::Round),
			..Default::default()
		};
		let child = SvgStyle {
			stroke_width: Some(2.0),
			fill: None,
			stroke: Some(SvgPaint::Color(SvgColor::new_rgb(0, 0, 255))),
			..Default::default()
		};

		// -- Exec
		let combined = child.inherit_from(&parent);

		// -- Check
		assert_eq!(combined.stroke_width, Some(2.0));
		assert_eq!(combined.fill, Some(SvgPaint::Color(SvgColor::new_rgb(255, 0, 0))));
		assert_eq!(combined.fill_opacity, Some(0.8));
		assert_eq!(combined.stroke, Some(SvgPaint::Color(SvgColor::new_rgb(0, 0, 255))));
		assert_eq!(combined.stroke_linecap, Some(StrokeLinecap::Round));

		Ok(())
	}

	#[test]
	fn test_ast_svg_view_box_1080p_scaling() -> Result<()> {
		// -- Setup & Fixtures
		let landscape = SvgViewBox::new(0.0, 0.0, 1920.0, 1080.0);
		let portrait = SvgViewBox::new(0.0, 0.0, 600.0, 1200.0);
		let square = SvgViewBox::new(0.0, 0.0, 200.0, 200.0);
		let zero = SvgViewBox::new(0.0, 0.0, 0.0, 0.0);

		// -- Exec
		let (land_w, land_h) = landscape.scaled_1080p_dimensions();
		let (port_w, port_h) = portrait.scaled_1080p_dimensions();
		let (sq_w, sq_h) = square.scaled_1080p_dimensions();
		let (z_w, z_h) = zero.scaled_1080p_dimensions();

		// -- Check
		assert_eq!(land_w, 1080.0);
		assert_eq!(land_h, 607.5);

		assert_eq!(port_w, 540.0);
		assert_eq!(port_h, 1080.0);

		assert_eq!(sq_w, 1080.0);
		assert_eq!(sq_h, 1080.0);

		assert_eq!(z_w, 1080.0);
		assert_eq!(z_h, 1080.0);

		Ok(())
	}

	#[test]
	fn test_ast_svg_text_element() -> Result<()> {
		// -- Setup & Fixtures
		let text = SvgText {
			id: Some("txt_1".to_string()),
			content: "Sample Text".to_string(),
			font_family: Some("Arial".to_string()),
			font_size: Some(24.0),
			..Default::default()
		};
		let mut element = SvgElement::Text(text);

		// -- Exec
		element.style_mut().stroke_width = Some(1.5);

		// -- Check
		assert_eq!(element.style().stroke_width, Some(1.5));
		if let SvgElement::Text(t) = element {
			assert_eq!(t.id.as_deref(), Some("txt_1"));
			assert_eq!(t.content, "Sample Text");
			assert_eq!(t.font_family.as_deref(), Some("Arial"));
			assert_eq!(t.font_size, Some(24.0));
		} else {
			return Err("Expected SvgElement::Text".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
