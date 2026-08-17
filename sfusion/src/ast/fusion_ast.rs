// region:    --- Types

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FusionOptions {
	pub end_with_stransform: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FusionDoc {
	pub tools: Vec<FusionTool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FusionTool {
	SPolygon(SPolygon),
	SMerge(SMerge),
	SText(SText),
	STransform(STransform),
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SText {
	pub name: String,
	pub styled_text: String,
	pub mask_width: Option<f64>,
	pub mask_height: Option<f64>,
	pub font: Option<String>,
	pub style: Option<String>,
	pub size: Option<f64>,
	pub line_spacing: Option<f64>,
	pub character_spacing: Option<f64>,
	pub red: Option<f64>,
	pub green: Option<f64>,
	pub blue: Option<f64>,
	pub opacity: Option<f64>,
	pub vertical_justification: Option<i32>,
	pub horizontal_justification: Option<i32>,
	pub horizontal_left_center_right: Option<i32>,
	pub wrap: Option<i32>,
	pub layout_rotation: Option<i32>,
	pub transform_rotation: Option<i32>,
	pub center_x: Option<f64>,
	pub center_y: Option<f64>,
	pub view_info: ViewInfo,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct STransform {
	pub name: String,
	pub input_op: Option<String>,
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

impl FusionOptions {
	pub fn with_end_with_stransform(mut self, end_with_stransform: bool) -> Self {
		self.end_with_stransform = end_with_stransform;
		self
	}
}

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

impl SText {
	pub fn new(name: impl Into<String>, styled_text: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			styled_text: styled_text.into(),
			..Default::default()
		}
	}
}

impl STransform {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			input_op: None,
			view_info: ViewInfo::default(),
		}
	}

	pub fn with_input_op(mut self, input_op: impl Into<String>) -> Self {
		self.input_op = Some(input_op.into());
		self
	}

	pub fn with_view_info(mut self, view_info: ViewInfo) -> Self {
		self.view_info = view_info;
		self
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

impl From<SText> for FusionTool {
	fn from(val: SText) -> Self {
		Self::SText(val)
	}
}

impl From<STransform> for FusionTool {
	fn from(val: STransform) -> Self {
		Self::STransform(val)
	}
}

// endregion: --- Froms

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_ast_fusion_stext_creation() -> Result<()> {
		// -- Setup & Fixtures
		let text_tool = SText {
			name: "sText1".to_string(),
			styled_text: "Hello Fusion".to_string(),
			mask_width: Some(1005.3947368421053),
			mask_height: Some(1080.0),
			font: Some("Open Sans".to_string()),
			style: Some("Bold".to_string()),
			size: Some(0.05),
			vertical_justification: Some(3),
			horizontal_justification: Some(3),
			view_info: ViewInfo::new(3520.0, -379.5),
			..Default::default()
		};

		// -- Exec
		let tool: FusionTool = text_tool.clone().into();

		// -- Check
		assert_eq!(tool, FusionTool::SText(text_tool));

		Ok(())
	}

	#[test]
	fn test_ast_fusion_options_builder() -> Result<()> {
		// -- Setup & Fixtures
		let options = FusionOptions::default().with_end_with_stransform(true);

		// -- Check
		assert!(options.end_with_stransform);

		Ok(())
	}

	#[test]
	fn test_ast_fusion_stransform_creation() -> Result<()> {
		// -- Setup & Fixtures
		let transform = STransform::new("sxf_1")
			.with_input_op("smerge")
			.with_view_info(ViewInfo::new(100.0, 200.0));

		// -- Exec
		let tool: FusionTool = transform.clone().into();

		// -- Check
		assert_eq!(tool, FusionTool::STransform(transform));

		Ok(())
	}
}

// endregion: --- Tests
