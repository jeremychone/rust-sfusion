use super::arc::arc_to_cubic_beziers;
use super::path_segment::{NormalizedSegment, Point};
use crate::error::{Error, Result};

// region:    --- Types

struct PathTokenizer<'a> {
	input: &'a str,
	chars: std::str::CharIndices<'a>,
}

// endregion: --- Types

// region:    --- Public Functions

/// Parses SVG path data `d` into a normalized sequence of segments.
pub fn parse_svg_path(d: &str) -> Result<Vec<NormalizedSegment>> {
	let mut tokenizer = PathTokenizer::new(d);
	let mut segments = Vec::new();

	let mut current_point = Point::new(0.0, 0.0);
	let mut subpath_start = Point::new(0.0, 0.0);
	let mut last_cubic_control: Option<Point> = None;
	let mut last_quad_control: Option<Point> = None;
	let mut current_command: Option<(char, bool)> = None;

	while let Some(ch) = tokenizer.next_command_or_lookahead() {
		let (cmd_char, is_relative) = match ch {
			'M' => ('M', false),
			'm' => ('m', true),
			'L' => ('L', false),
			'l' => ('l', true),
			'H' => ('H', false),
			'h' => ('h', true),
			'V' => ('V', false),
			'v' => ('v', true),
			'C' => ('C', false),
			'c' => ('c', true),
			'S' => ('S', false),
			's' => ('s', true),
			'Q' => ('Q', false),
			'q' => ('q', true),
			'T' => ('T', false),
			't' => ('t', true),
			'A' => ('A', false),
			'a' => ('a', true),
			'Z' | 'z' => {
				segments.push(NormalizedSegment::Close);
				current_point = subpath_start;
				last_cubic_control = None;
				last_quad_control = None;
				current_command = None;
				continue;
			}
			_ => {
				if let Some(cmd) = current_command {
					cmd
				} else {
					return Err(Error::custom(format!("Unexpected character in path: {ch}")));
				}
			}
		};

		current_command = Some((cmd_char, is_relative));

		match cmd_char.to_ascii_uppercase() {
			'M' => {
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for MoveTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for MoveTo"))?;

				let pt = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				segments.push(NormalizedSegment::MoveTo(pt));
				current_point = pt;
				subpath_start = pt;
				last_cubic_control = None;
				last_quad_control = None;

				// Subsequent coordinate pairs after M/m are treated as implicit L/l
				current_command = Some((if is_relative { 'l' } else { 'L' }, is_relative));
			}

			'L' => {
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for LineTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for LineTo"))?;

				let pt = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				segments.push(NormalizedSegment::LineTo(pt));
				current_point = pt;
				last_cubic_control = None;
				last_quad_control = None;
			}

			'H' => {
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for HorizontalLineTo"))?;
				let pt = if is_relative {
					Point::new(current_point.x + x, current_point.y)
				} else {
					Point::new(x, current_point.y)
				};

				segments.push(NormalizedSegment::LineTo(pt));
				current_point = pt;
				last_cubic_control = None;
				last_quad_control = None;
			}

			'V' => {
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for VerticalLineTo"))?;
				let pt = if is_relative {
					Point::new(current_point.x, current_point.y + y)
				} else {
					Point::new(current_point.x, y)
				};

				segments.push(NormalizedSegment::LineTo(pt));
				current_point = pt;
				last_cubic_control = None;
				last_quad_control = None;
			}

			'C' => {
				let x1 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X1 for CubicTo"))?;
				let y1 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y1 for CubicTo"))?;
				let x2 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X2 for CubicTo"))?;
				let y2 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y2 for CubicTo"))?;
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for CubicTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for CubicTo"))?;

				let p1 = if is_relative {
					Point::new(current_point.x + x1, current_point.y + y1)
				} else {
					Point::new(x1, y1)
				};
				let p2 = if is_relative {
					Point::new(current_point.x + x2, current_point.y + y2)
				} else {
					Point::new(x2, y2)
				};
				let p = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				segments.push(NormalizedSegment::CubicTo { p1, p2, p });
				last_cubic_control = Some(p2);
				last_quad_control = None;
				current_point = p;
			}

			'S' => {
				let x2 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X2 for SmoothCubicTo"))?;
				let y2 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y2 for SmoothCubicTo"))?;
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for SmoothCubicTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for SmoothCubicTo"))?;

				let p1 = if let Some(last_cp) = last_cubic_control {
					Point::new(2.0 * current_point.x - last_cp.x, 2.0 * current_point.y - last_cp.y)
				} else {
					current_point
				};

				let p2 = if is_relative {
					Point::new(current_point.x + x2, current_point.y + y2)
				} else {
					Point::new(x2, y2)
				};
				let p = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				segments.push(NormalizedSegment::CubicTo { p1, p2, p });
				last_cubic_control = Some(p2);
				last_quad_control = None;
				current_point = p;
			}

			'Q' => {
				let x1 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X1 for QuadTo"))?;
				let y1 = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y1 for QuadTo"))?;
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for QuadTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for QuadTo"))?;

				let q_cp = if is_relative {
					Point::new(current_point.x + x1, current_point.y + y1)
				} else {
					Point::new(x1, y1)
				};
				let p = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				let p1 = Point::new(
					current_point.x + (2.0 / 3.0) * (q_cp.x - current_point.x),
					current_point.y + (2.0 / 3.0) * (q_cp.y - current_point.y),
				);
				let p2 = Point::new(
					p.x + (2.0 / 3.0) * (q_cp.x - p.x),
					p.y + (2.0 / 3.0) * (q_cp.y - p.y),
				);

				segments.push(NormalizedSegment::CubicTo { p1, p2, p });
				last_quad_control = Some(q_cp);
				last_cubic_control = None;
				current_point = p;
			}

			'T' => {
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for SmoothQuadTo"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for SmoothQuadTo"))?;

				let q_cp = if let Some(last_qp) = last_quad_control {
					Point::new(2.0 * current_point.x - last_qp.x, 2.0 * current_point.y - last_qp.y)
				} else {
					current_point
				};

				let p = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				let p1 = Point::new(
					current_point.x + (2.0 / 3.0) * (q_cp.x - current_point.x),
					current_point.y + (2.0 / 3.0) * (q_cp.y - current_point.y),
				);
				let p2 = Point::new(
					p.x + (2.0 / 3.0) * (q_cp.x - p.x),
					p.y + (2.0 / 3.0) * (q_cp.y - p.y),
				);

				segments.push(NormalizedSegment::CubicTo { p1, p2, p });
				last_quad_control = Some(q_cp);
				last_cubic_control = None;
				current_point = p;
			}

			'A' => {
				let rx = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected rx for Arc"))?;
				let ry = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected ry for Arc"))?;
				let x_rot = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected rotation for Arc"))?;
				let large_arc = tokenizer.next_flag()?.ok_or_else(|| Error::custom("Expected large arc flag"))?;
				let sweep = tokenizer.next_flag()?.ok_or_else(|| Error::custom("Expected sweep flag"))?;
				let x = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected X for Arc"))?;
				let y = tokenizer.next_number()?.ok_or_else(|| Error::custom("Expected Y for Arc"))?;

				let end_pt = if is_relative {
					Point::new(current_point.x + x, current_point.y + y)
				} else {
					Point::new(x, y)
				};

				let arc_segments = arc_to_cubic_beziers(current_point, rx, ry, x_rot, large_arc, sweep, end_pt);
				segments.extend(arc_segments);

				current_point = end_pt;
				last_cubic_control = None;
				last_quad_control = None;
			}

			_ => {}
		}
	}

	Ok(segments)
}

// endregion: --- Public Functions

// region:    --- Support

impl<'a> PathTokenizer<'a> {
	fn new(input: &'a str) -> Self {
		Self {
			input,
			chars: input.char_indices(),
		}
	}

	fn skip_whitespace_and_commas(&mut self) {
		while let Some((_, ch)) = self.peek_char() {
			if ch.is_whitespace() || ch == ',' {
				self.chars.next();
			} else {
				break;
			}
		}
	}

	fn peek_char(&self) -> Option<(usize, char)> {
		self.chars.clone().next()
	}

	fn next_command_or_lookahead(&mut self) -> Option<char> {
		self.skip_whitespace_and_commas();
		let (_, ch) = self.peek_char()?;

		if is_command_char(ch) {
			self.chars.next();
			Some(ch)
		} else {
			Some(ch)
		}
	}

	fn next_flag(&mut self) -> Result<Option<bool>> {
		self.skip_whitespace_and_commas();
		let (_, ch) = match self.peek_char() {
			Some(c) => c,
			None => return Ok(None),
		};

		match ch {
			'0' => {
				self.chars.next();
				Ok(Some(false))
			}
			'1' => {
				self.chars.next();
				Ok(Some(true))
			}
			_ => Err(Error::custom(format!("Invalid flag character: {ch}"))),
		}
	}

	fn next_number(&mut self) -> Result<Option<f64>> {
		self.skip_whitespace_and_commas();
		let (start_idx, first_ch) = match self.peek_char() {
			Some(c) => c,
			None => return Ok(None),
		};

		if is_command_char(first_ch) {
			return Ok(None);
		}

		let mut end_idx = start_idx;
		let mut has_digit = false;
		let mut has_dot = false;
		let mut has_exp = false;

		while let Some((idx, ch)) = self.peek_char() {
			if ch == '+' || ch == '-' {
				if idx == start_idx || (has_exp && end_idx == idx) {
					end_idx = idx + ch.len_utf8();
					self.chars.next();
				} else {
					break;
				}
			} else if ch == '.' {
				if has_dot || has_exp {
					break;
				}
				has_dot = true;
				end_idx = idx + ch.len_utf8();
				self.chars.next();
			} else if ch == 'e' || ch == 'E' {
				if has_exp || !has_digit {
					break;
				}
				has_exp = true;
				end_idx = idx + ch.len_utf8();
				self.chars.next();
			} else if ch.is_ascii_digit() {
				has_digit = true;
				end_idx = idx + ch.len_utf8();
				self.chars.next();
			} else {
				break;
			}
		}

		if !has_digit {
			return Ok(None);
		}

		let num_str = &self.input[start_idx..end_idx];
		let val = num_str
			.parse::<f64>()
			.map_err(|e| Error::custom(format!("Failed to parse number '{num_str}': {e}")))?;

		Ok(Some(val))
	}
}

fn is_command_char(ch: char) -> bool {
	matches!(
		ch,
		'M' | 'm'
			| 'L' | 'l'
			| 'H' | 'h'
			| 'V' | 'v'
			| 'C' | 'c'
			| 'S' | 's'
			| 'Q' | 'q'
			| 'T' | 't'
			| 'A' | 'a'
			| 'Z' | 'z'
	)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_svg_path_parser_simple_triangle() -> Result<()> {
		// -- Setup & Fixtures
		let d = "M 10 20 L 30 40 L 50 20 Z";

		// -- Exec
		let segments = parse_svg_path(d)?;

		// -- Check
		assert_eq!(segments.len(), 4);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, 20.0)));
		assert_eq!(segments[1], NormalizedSegment::LineTo(Point::new(30.0, 40.0)));
		assert_eq!(segments[2], NormalizedSegment::LineTo(Point::new(50.0, 20.0)));
		assert_eq!(segments[3], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_path_parser_relative_and_implicit_lines() -> Result<()> {
		// -- Setup & Fixtures
		let d = "m 10 20 20 20 -10 5 z";

		// -- Exec
		let segments = parse_svg_path(d)?;

		// -- Check
		assert_eq!(segments.len(), 4);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, 20.0)));
		assert_eq!(segments[1], NormalizedSegment::LineTo(Point::new(30.0, 40.0)));
		assert_eq!(segments[2], NormalizedSegment::LineTo(Point::new(20.0, 45.0)));
		assert_eq!(segments[3], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_path_parser_cubic_and_smooth() -> Result<()> {
		// -- Setup & Fixtures
		let d = "M 0 0 C 10 10 20 10 30 0 S 50 -10 60 0";

		// -- Exec
		let segments = parse_svg_path(d)?;

		// -- Check
		assert_eq!(segments.len(), 3);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(0.0, 0.0)));
		assert_eq!(
			segments[1],
			NormalizedSegment::CubicTo {
				p1: Point::new(10.0, 10.0),
				p2: Point::new(20.0, 10.0),
				p: Point::new(30.0, 0.0),
			}
		);
		// S first control point is reflection of (20, 10) across (30, 0) => (40, -10)
		assert_eq!(
			segments[2],
			NormalizedSegment::CubicTo {
				p1: Point::new(40.0, -10.0),
				p2: Point::new(50.0, -10.0),
				p: Point::new(60.0, 0.0),
			}
		);

		Ok(())
	}

	#[test]
	fn test_svg_path_parser_compact_numbers() -> Result<()> {
		// -- Setup & Fixtures
		let d = "M10-20.5.3-4.5e1Z";

		// -- Exec
		let segments = parse_svg_path(d)?;

		// -- Check
		assert_eq!(segments.len(), 3);
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, -20.5)));
		assert_eq!(segments[1], NormalizedSegment::LineTo(Point::new(0.3, -45.0)));
		assert_eq!(segments[2], NormalizedSegment::Close);

		Ok(())
	}

	#[test]
	fn test_svg_path_parser_arc_flags() -> Result<()> {
		// -- Setup & Fixtures
		let d = "M 10 80 A 45 45 0 0 0 125 125";

		// -- Exec
		let segments = parse_svg_path(d)?;

		// -- Check
		assert!(!segments.is_empty());
		assert_eq!(segments[0], NormalizedSegment::MoveTo(Point::new(10.0, 80.0)));
		if let NormalizedSegment::CubicTo { p, .. } = segments.last().ok_or("segment missing")? {
			assert!((p.x - 125.0).abs() < 1e-6);
			assert!((p.y - 125.0).abs() < 1e-6);
		} else {
			return Err("Expected cubic segment at end of arc".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
