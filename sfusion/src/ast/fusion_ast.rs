// region:    --- Types

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FusionDoc {
	pub tools: Vec<FusionTool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FusionTool {
	SPolygon(SPolygon),
	SMerge(SMerge),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SPolygon {
	pub name: String,
	pub mask_width: f64,
	pub mask_height: f64,
	pub border_width: Option<f64>,
	pub red: Option<f64>,
	pub green: Option<f64>,
	pub blue: Option<f64>,
	pub opacity: Option<f64>,
	pub points: Vec<PolylinePoint>,
	pub closed: bool,
	pub view_info: ViewInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SMerge {
	pub name: String,
	pub inputs: Vec<String>,
	pub view_info: ViewInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolylinePoint {
	pub x: f64,
	pub y: f64,
	pub lx: f64,
	pub ly: f64,
	pub rx: f64,
	pub ry: f64,
	pub linear: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewInfo {
	pub pos_x: f64,
	pub pos_y: f64,
}

// endregion: --- Types

// region:    --- Constructors

impl PolylinePoint {
	pub fn new_linear(x: f64, y: f64) -> Self {
		Self {
			x,
			y,
			lx: 0.0,
			ly: 0.0,
			rx: 0.0,
			ry: 0.0,
			linear: true,
		}
	}

	pub fn new_smooth(x: f64, y: f64, lx: f64, ly: f64, rx: f64, ry: f64) -> Self {
		Self {
			x,
			y,
			lx,
			ly,
			rx,
			ry,
			linear: false,
		}
	}
}

impl ViewInfo {
	pub fn new(pos_x: f64, pos_y: f64) -> Self {
		Self { pos_x, pos_y }
	}
}

// endregion: --- Constructors

// region:    --- Froms

impl From<SPolygon> for FusionTool {
	fn from(val: SPolygon) -> Self {
		Self::SPolygon(val)
	}
}

impl From<SMerge> for FusionTool {
	fn from(val: SMerge) -> Self {
		Self::SMerge(val)
	}
}

// endregion: --- Froms
