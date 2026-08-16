use super::naming::NameTracker;
use super::polyline::segments_to_polylines;
use crate::ast::*;
use crate::error::Result;
use crate::svg::{element_to_segments, NormalizedSegment, Point};

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
}

// endregion: --- Types

// region:    --- Public Functions

/// Converts an `SvgDoc` into a `FusionDoc` graph with positioned tools and merges.
pub fn build_fusion_doc(svg_doc: &SvgDoc) -> Result<FusionDoc> {
	let mut builder = GraphBuilder::default();
	let view_box = svg_doc.effective_view_box();

	let mut top_output_names = Vec::new();

	for element in &svg_doc.elements {
		if let Some(out_name) = builder.process_element(element, &view_box, Transform2D::identity(), &SvgStyle::default())? {
			top_output_names.push(out_name);
		}
	}

	// Sort sPolygon tools alphabetically, followed by sMerge tools
	builder.tools.sort_by(|a, b| match (a, b) {
		(FusionTool::SPolygon(p1), FusionTool::SPolygon(p2)) => p1.name.cmp(&p2.name),
		(FusionTool::SPolygon(_), FusionTool::SMerge(_)) => std::cmp::Ordering::Less,
		(FusionTool::SMerge(_), FusionTool::SPolygon(_)) => std::cmp::Ordering::Greater,
		(FusionTool::SMerge(m1), FusionTool::SMerge(m2)) => m1.name.cmp(&m2.name),
	});

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
		let pos_y = DEFAULT_START_Y;
		self.col_counter += 1;
		ViewInfo::new(pos_x, pos_y)
	}

	fn next_merge_pos(&mut self) -> ViewInfo {
		let pos_x = DEFAULT_START_X + ((self.col_counter.saturating_sub(1)) as f64) * GRID_STEP_X;
		let pos_y = DEFAULT_START_Y + GRID_STEP_Y;
		ViewInfo::new(pos_x, pos_y)
	}

	fn process_element(
		&mut self,
		element: &SvgElement,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Option<String>> {
		match element {
			SvgElement::Group(group) => self.process_group(group, view_box, parent_tf, parent_style),
			_ => self.process_shape(element, view_box, parent_tf, parent_style),
		}
	}

	fn process_shape(
		&mut self,
		element: &SvgElement,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Option<String>> {
		let elem_tf = get_element_transform(element).unwrap_or_default();
		let total_tf = parent_tf.multiply(&elem_tf);

		let raw_segments = element_to_segments(element)?;
		let transformed_segments: Vec<NormalizedSegment> = raw_segments
			.into_iter()
			.map(|seg| transform_segment(seg, total_tf))
			.collect();

		let polylines = segments_to_polylines(&transformed_segments, view_box);
		if polylines.is_empty() {
			return Ok(None);
		}

		let explicit_id = get_element_id(element);
		let effective_style = element.style().inherit_from(parent_style);
		let border_width = effective_style.stroke_width.map(|sw| {
			let denom = if view_box.width == 0.0 { 1.0 } else { view_box.width };
			sw / denom
		});
		let mut last_name = None;

		for poly in polylines {
			let name = self.name_tracker.generate_unique_name(explicit_id);
			let pos = self.next_leaf_pos();

			let spolygon = SPolygon {
				name: name.clone(),
				mask_width: 320.0,
				mask_height: 240.0,
				border_width,
				points: poly.points,
				closed: poly.closed,
				view_info: pos,
			};

			self.tools.push(FusionTool::SPolygon(spolygon));
			last_name = Some(name);
		}

		Ok(last_name)
	}

	fn process_group(
		&mut self,
		group: &SvgGroup,
		view_box: &SvgViewBox,
		parent_tf: Transform2D,
		parent_style: &SvgStyle,
	) -> Result<Option<String>> {
		let group_tf = group.transform.unwrap_or_default();
		let total_tf = parent_tf.multiply(&group_tf);
		let effective_style = group.style.inherit_from(parent_style);

		let mut child_names = Vec::new();

		for child in &group.children {
			if let Some(name) = self.process_element(child, view_box, total_tf, &effective_style)? {
				child_names.push(name);
			}
		}

		if child_names.is_empty() {
			return Ok(None);
		}

		if child_names.len() == 1 {
			let mut child_name = child_names.remove(0);
			if let Some(group_id) = group.id.as_deref()
				&& !group_id.trim().is_empty()
			{
				let new_name = self.name_tracker.generate_unique_name(Some(group_id));
				for tool in &mut self.tools {
					match tool {
						FusionTool::SPolygon(poly) if poly.name == child_name => {
							poly.name = new_name.clone();
						}
						FusionTool::SMerge(merge) if merge.name == child_name => {
							merge.name = new_name.clone();
						}
						_ => {}
					}
					if let FusionTool::SMerge(merge) = tool {
						for input in &mut merge.inputs {
							if *input == child_name {
								*input = new_name.clone();
							}
						}
					}
				}
				child_name = new_name;
			}
			return Ok(Some(child_name));
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

fn transform_segment(seg: NormalizedSegment, tf: Transform2D) -> NormalizedSegment {
	match seg {
		NormalizedSegment::MoveTo(p) => {
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::MoveTo(Point::new(x, y))
		}
		NormalizedSegment::LineTo(p) => {
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::LineTo(Point::new(x, y))
		}
		NormalizedSegment::CubicTo { p1, p2, p } => {
			let (x1, y1) = tf.transform_xy(p1.x, p1.y);
			let (x2, y2) = tf.transform_xy(p2.x, p2.y);
			let (x, y) = tf.transform_xy(p.x, p.y);
			NormalizedSegment::CubicTo {
				p1: Point::new(x1, y1),
				p2: Point::new(x2, y2),
				p: Point::new(x, y),
			}
		}
		NormalizedSegment::Close => NormalizedSegment::Close,
	}
}

fn get_element_transform(element: &SvgElement) -> Option<Transform2D> {
	match element {
		SvgElement::Path(p) => p.transform,
		SvgElement::Rect(r) => r.transform,
		SvgElement::Circle(c) => c.transform,
		SvgElement::Ellipse(e) => e.transform,
		SvgElement::Line(l) => l.transform,
		SvgElement::Polyline(pl) => pl.transform,
		SvgElement::Polygon(pg) => pg.transform,
		SvgElement::Group(g) => g.transform,
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
			defs: SvgDefs::default(),
			elements: vec![
				SvgElement::Path(SvgPath {
					id: Some("poly_1".to_string()),
					transform: None,
					style: SvgStyle::default(),
					d: "M 10 20 L 30 40 Z".to_string(),
				}),
				SvgElement::Rect(SvgRect {
					id: Some("grabber".to_string()),
					transform: None,
					style: SvgStyle::default(),
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
			assert_eq!(p1.name, "grabber");
			assert_eq!(p1.view_info.pos_x, 2090.0);
			assert_eq!(p1.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon grabber as first tool".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "poly_1");
			assert_eq!(p2.view_info.pos_x, 1980.0);
			assert_eq!(p2.view_info.pos_y, -247.5);
		} else {
			return Err("Expected SPolygon poly_1 as second tool".into());
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

	#[test]
	fn test_fusion_graph_builder_single_child_group_inherits_id() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 320.0, 240.0)),
			width: Some(320.0),
			height: Some(240.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("grabber".to_string()),
				transform: None,
				style: SvgStyle::default(),
				children: vec![SvgElement::Circle(SvgCircle {
					id: None,
					transform: None,
					style: SvgStyle::default(),
					cx: 50.0,
					cy: 50.0,
					r: 25.0,
				})],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 1);
		if let FusionTool::SPolygon(p) = &fusion_doc.tools[0] {
			assert_eq!(p.name, "grabber");
			assert_eq!(p.border_width, None);
		} else {
			return Err("Expected SPolygon grabber".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_stroke_width_inheritance() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 200.0, 100.0)),
			width: Some(200.0),
			height: Some(100.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("styled_group".to_string()),
				transform: None,
				style: SvgStyle {
					stroke_width: Some(4.0),
					..Default::default()
				},
				children: vec![
					SvgElement::Path(SvgPath {
						id: Some("inherited_path".to_string()),
						transform: None,
						style: SvgStyle::default(),
						d: "M 0 0 L 10 10".to_string(),
					}),
					SvgElement::Path(SvgPath {
						id: Some("override_path".to_string()),
						transform: None,
						style: SvgStyle {
							stroke_width: Some(10.0),
							..Default::default()
						},
						d: "M 10 10 L 20 20".to_string(),
					}),
				],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "inherited_path");
			assert_eq!(p1.border_width, Some(4.0 / 200.0));
		} else {
			return Err("Expected inherited_path SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "override_path");
			assert_eq!(p2.border_width, Some(10.0 / 200.0));
		} else {
			return Err("Expected override_path SPolygon".into());
		}

		Ok(())
	}

	#[test]
	fn test_fusion_graph_builder_deep_nested_group_styles() -> Result<()> {
		// -- Setup & Fixtures
		let svg_doc = SvgDoc {
			view_box: Some(SvgViewBox::new(0.0, 0.0, 400.0, 200.0)),
			width: Some(400.0),
			height: Some(200.0),
			defs: SvgDefs::default(),
			elements: vec![SvgElement::Group(SvgGroup {
				id: Some("root_group".to_string()),
				transform: None,
				style: SvgStyle {
					stroke_width: Some(8.0),
					..Default::default()
				},
				children: vec![SvgElement::Group(SvgGroup {
					id: Some("inner_group".to_string()),
					transform: None,
					style: SvgStyle::default(),
					children: vec![
						SvgElement::Rect(SvgRect {
							id: Some("rect1".to_string()),
							transform: None,
							style: SvgStyle::default(),
							x: 0.0,
							y: 0.0,
							width: 50.0,
							height: 50.0,
							rx: None,
							ry: None,
						}),
						SvgElement::Rect(SvgRect {
							id: Some("rect2".to_string()),
							transform: None,
							style: SvgStyle {
								stroke_width: Some(2.0),
								..Default::default()
							},
							x: 60.0,
							y: 0.0,
							width: 50.0,
							height: 50.0,
							rx: None,
							ry: None,
						}),
					],
				})],
			})],
		};

		// -- Exec
		let fusion_doc = build_fusion_doc(&svg_doc)?;

		// -- Check
		assert_eq!(fusion_doc.tools.len(), 3);
		if let FusionTool::SPolygon(p1) = &fusion_doc.tools[0] {
			assert_eq!(p1.name, "rect1");
			assert_eq!(p1.border_width, Some(8.0 / 400.0));
		} else {
			return Err("Expected rect1 SPolygon".into());
		}

		if let FusionTool::SPolygon(p2) = &fusion_doc.tools[1] {
			assert_eq!(p2.name, "rect2");
			assert_eq!(p2.border_width, Some(2.0 / 400.0));
		} else {
			return Err("Expected rect2 SPolygon".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
