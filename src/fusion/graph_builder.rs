use super::naming::NameTracker;
use super::polyline::element_to_polylines;
use crate::ast::*;
use crate::error::Result;

// Layout grid constants for DaVinci Resolve Fusion operator spacing
const DEFAULT_START_X: f64 = 1980.0;
const DEFAULT_START_Y: f64 = -247.5;
const GRID_STEP_X: f64 = 110.0;
const GRID_STEP_Y: f64 = 66.0;

// region:    --- Types

#[derive(Default)]
pub struct GraphBuilder {
	name_tracker: NameTracker,
	tools: Vec<FusionTool>,
	col_counter: usize,
	row_level: usize,
}

// endregion: --- Types

// region:    --- Public Functions

/// Converts an `SvgDoc` into a `FusionDoc` graph with positioned tools and merges.
pub fn build_fusion_doc(svg_doc: &SvgDoc) -> Result<FusionDoc> {
	let mut builder = GraphBuilder::default();
	let view_box = svg_doc.effective_view_box();

	let mut top_output_names = Vec::new();

	for element in &svg_doc.elements {
		if let Some(out_name) = builder.process_element(element, &view_box)? {
			top_output_names.push(out_name);
		}
	}

	// If there are multiple top-level elements without a containing group, merge them
	if top_output_names.len() > 1 {
		let merge_name = builder.name_tracker.generate_unique_name(Some("loop"));
		let pos = builder.next_merge_pos();
		let s_merge = SMerge {
			name: merge_name,
			inputs: top_output_names,
			view_info: pos,
		};
		builder.tools.push(FusionTool::SMerge(s_merge));
	}

	Ok(FusionDoc { tools: builder.tools })
}

// endregion: --- Public Functions

// region:    --- Support

impl GraphBuilder {
	fn next_leaf_pos(&mut self) -> ViewInfo {
		let pos_x = DEFAULT_START_X + (self.col_counter as f64) * GRID_STEP_X;
		let pos_y = DEFAULT_START_Y - (self.row_level as f64) * GRID_STEP_Y;
		self.col_counter += 1;
		ViewInfo::new(pos_x, pos_y)
	}

	fn next_merge_pos(&mut self) -> ViewInfo {
		let pos_x = DEFAULT_START_X + ((self.col_counter.saturating_sub(1)) as f64) * GRID_STEP_X;
		let pos_y = DEFAULT_START_Y + GRID_STEP_Y;
		ViewInfo::new(pos_x, pos_y)
	}

	fn process_element(&mut self, element: &SvgElement, view_box: &SvgViewBox) -> Result<Option<String>> {
		match element {
			SvgElement::Group(group) => self.process_group(group, view_box),
			_ => self.process_shape(element, view_box),
		}
	}

	fn process_shape(&mut self, element: &SvgElement, view_box: &SvgViewBox) -> Result<Option<String>> {
		let polylines = element_to_polylines(element, view_box)?;
		if polylines.is_empty() {
			return Ok(None);
		}

		let explicit_id = get_element_id(element);
		let mut last_name = None;

		for poly in polylines {
			let name = self.name_tracker.generate_unique_name(explicit_id);
			let pos = self.next_leaf_pos();

			let spolygon = SPolygon {
				name: name.clone(),
				mask_width: view_box.width,
				mask_height: view_box.height,
				points: poly.points,
				closed: poly.closed,
				view_info: pos,
			};

			self.tools.push(FusionTool::SPolygon(spolygon));
			last_name = Some(name);
		}

		Ok(last_name)
	}

	fn process_group(&mut self, group: &SvgGroup, view_box: &SvgViewBox) -> Result<Option<String>> {
		self.row_level += 1;
		let mut child_names = Vec::new();

		for child in &group.children {
			if let Some(name) = self.process_element(child, view_box)? {
				child_names.push(name);
			}
		}

		self.row_level -= 1;

		if child_names.is_empty() {
			return Ok(None);
		}

		if child_names.len() == 1 {
			return Ok(Some(child_names.remove(0)));
		}

		let group_id = group.id.as_deref().or(Some("loop"));
		let merge_name = self.name_tracker.generate_unique_name(group_id);
		let pos = self.next_merge_pos();

		let s_merge = SMerge {
			name: merge_name.clone(),
			inputs: child_names,
			view_info: pos,
		};

		self.tools.push(FusionTool::SMerge(s_merge));
		Ok(Some(merge_name))
	}
}

fn get_element_id(element: &SvgElement) -> Option<&str> {
	match element {
		SvgElement::Path(p) => p.id.as_deref(),
		SvgElement::Rect(r) => r.id.as_deref(),
		SvgElement::Circle(c) => c.id.as_deref(),
		SvgElement::Ellipse(e) => e.id.as_deref(),
		SvgElement::Line(l) => l.id.as_deref(),
		SvgElement::Polyline(pl) => pl.id.as_deref(),
		SvgElement::Polygon(pg) => pg.id.as_deref(),
		SvgElement::Group(g) => g.id.as_deref(),
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_fusion_graph_builder_two_shapes() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 320.0, 240.0)),
			width: Some(320.0),
			height: Some(240.0),
			elements: vec![
				SvgElement::Path(SvgPath {
					id: Some("poly_1".to_string()),
					d: "M 10 20 L 30 40 Z".to_string(),
				}),
				SvgElement::Rect(SvgRect {
					id: Some("grabber".to_string()),
					x: 10.0,
					y: 20.0,
					width: 50.0,
					height: 60.0,
					rx: None,
					ry: None,
				}),
			],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);

		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "poly_1");
			assert_eq!(p1.view_info.pos_x, 1980.0);
			assert_eq!(p1.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon as first tool".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "grabber");
			assert_eq!(p2.view_info.pos_x, 2090.0);
			assert_eq!(p2.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon as second tool".into());
		}

		if let FusionTool::SMerge(m) = &fusion_doc.tools[2] {
			assert_eq!(m.name, "loop");
			assert_eq!(m.inputs, vec!["poly_1", "grabber"]);
			assert_eq!(m.view_info.pos_x, 2090.0);
			assert_eq!(m.view_info.pos_y, -181.5);
		} else {
			return Err("Expected SMerge as third tool".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
