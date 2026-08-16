use super::path_segment::{NormalizedSegment, Point};
use std::f64::consts::PI;

// region:    --- Public Functions

/// Converts an SVG elliptical arc command into a series of cubic bezier curve segments.
pub fn arc_to_cubic_beziers(
	start: Point,
	rx: f64,
	ry: f64,
	x_axis_rotation_deg: f64,
	large_arc_flag: bool,
	sweep_flag: bool,
	end: Point,
) -> Vec<NormalizedSegment> {
	// If the endpoints are identical, no arc is drawn.
	if (start.x - end.x).abs() < f64::EPSILON && (start.y - end.y).abs() < f64::EPSILON {
		return Vec::new();
	}

	let mut rx = rx.abs();
	let mut ry = ry.abs();

	// If either radius is 0, the result is treated as a straight line.
	if rx < f64::EPSILON || ry < f64::EPSILON {
		return vec![NormalizedSegment::LineTo(end)];
	}

	let phi = x_axis_rotation_deg.to_radians();
	let cos_phi = phi.cos();
	let sin_phi = phi.sin();

	// Step 1: Compute (x1_prime, y1_prime)
	let dx = (start.x - end.x) / 2.0;
	let dy = (start.y - end.y) / 2.0;

	let x1_prime = cos_phi * dx + sin_phi * dy;
	let y1_prime = -sin_phi * dx + cos_phi * dy;

	// Ensure radii are large enough
	let lambda = (x1_prime * x1_prime) / (rx * rx) + (y1_prime * y1_prime) / (ry * ry);
	if lambda > 1.0 {
		let lambda_sqrt = lambda.sqrt();
		rx *= lambda_sqrt;
		ry *= lambda_sqrt;
	}

	// Step 2: Compute (cx_prime, cy_prime)
	let rx_sq = rx * rx;
	let ry_sq = ry * ry;
	let x1_prime_sq = x1_prime * x1_prime;
	let y1_prime_sq = y1_prime * y1_prime;

	let numerator = rx_sq * ry_sq - rx_sq * y1_prime_sq - ry_sq * x1_prime_sq;
	let denominator = rx_sq * y1_prime_sq + ry_sq * x1_prime_sq;

	let sq = if denominator > 0.0 {
		(numerator / denominator).max(0.0)
	} else {
		0.0
	};

	let sign = if large_arc_flag == sweep_flag {
		-1.0
	} else {
		1.0
	};
	let coef = sign * sq.sqrt();

	let cx_prime = coef * (rx * y1_prime) / ry;
	let cy_prime = coef * -(ry * x1_prime) / rx;

	// Step 3: Compute (cx, cy) from (cx_prime, cy_prime)
	let cx = cos_phi * cx_prime - sin_phi * cy_prime + (start.x + end.x) / 2.0;
	let cy = sin_phi * cx_prime + cos_phi * cy_prime + (start.y + end.y) / 2.0;

	// Step 4: Compute theta1 and delta_theta
	let ux = (x1_prime - cx_prime) / rx;
	let uy = (y1_prime - cy_prime) / ry;
	let vx = (-x1_prime - cx_prime) / rx;
	let vy = (-y1_prime - cy_prime) / ry;

	let theta1 = angle_between(1.0, 0.0, ux, uy);
	let mut delta_theta = angle_between(ux, uy, vx, vy);

	if !sweep_flag && delta_theta > 0.0 {
		delta_theta -= 2.0 * PI;
	} else if sweep_flag && delta_theta < 0.0 {
		delta_theta += 2.0 * PI;
	}

	// Step 5: Subdivide into segments of at most PI / 2
	let segments_count = ((delta_theta.abs() / (PI / 2.0)).ceil() as usize).max(1);
	let segment_delta = delta_theta / segments_count as f64;

	let mut result = Vec::with_capacity(segments_count);
	let mut current_theta = theta1;

	for i in 0..segments_count {
		let next_theta = current_theta + segment_delta;
		let is_last = i == segments_count - 1;

		let segment_end = if is_last {
			end
		} else {
			point_on_ellipse(cx, cy, rx, ry, phi, next_theta)
		};

		let d_theta = next_theta - current_theta;
		let alpha = (4.0 / 3.0) * (d_theta / 4.0).tan();

		let d1 = ellipse_derivative(rx, ry, phi, current_theta);
		let d2 = ellipse_derivative(rx, ry, phi, next_theta);

		let p_curr = if i == 0 {
			start
		} else {
			point_on_ellipse(cx, cy, rx, ry, phi, current_theta)
		};

		let cp1 = Point::new(p_curr.x + alpha * d1.x, p_curr.y + alpha * d1.y);
		let cp2 = Point::new(segment_end.x - alpha * d2.x, segment_end.y - alpha * d2.y);

		result.push(NormalizedSegment::CubicTo {
			p1: cp1,
			p2: cp2,
			p: segment_end,
		});

		current_theta = next_theta;
	}

	result
}

// endregion: --- Public Functions

// region:    --- Support

fn angle_between(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
	let dot = ux * vx + uy * vy;
	let len_u = (ux * ux + uy * uy).sqrt();
	let len_v = (vx * vx + vy * vy).sqrt();
	let cos_val = (dot / (len_u * len_v)).clamp(-1.0, 1.0);
	let angle = cos_val.acos();
	if ux * vy - uy * vx < 0.0 {
		-angle
	} else {
		angle
	}
}

fn point_on_ellipse(cx: f64, cy: f64, rx: f64, ry: f64, phi: f64, theta: f64) -> Point {
	let cos_phi = phi.cos();
	let sin_phi = phi.sin();
	let cos_t = theta.cos();
	let sin_t = theta.sin();

	Point::new(
		cx + rx * cos_t * cos_phi - ry * sin_t * sin_phi,
		cy + rx * cos_t * sin_phi + ry * sin_t * cos_phi,
	)
}

fn ellipse_derivative(rx: f64, ry: f64, phi: f64, theta: f64) -> Point {
	let cos_phi = phi.cos();
	let sin_phi = phi.sin();
	let cos_t = theta.cos();
	let sin_t = theta.sin();

	Point::new(
		-rx * sin_t * cos_phi - ry * cos_t * sin_phi,
		-rx * sin_t * sin_phi + ry * cos_t * cos_phi,
	)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_svg_arc_zero_radius() -> Result<()> {
		// -- Setup & Fixtures
		let start = Point::new(0.0, 0.0);
		let end = Point::new(10.0, 10.0);

		// -- Exec
		let segs = arc_to_cubic_beziers(start, 0.0, 0.0, 0.0, false, false, end);

		// -- Check
		assert_eq!(segs.len(), 1);
		assert_eq!(segs[0], NormalizedSegment::LineTo(end));
		Ok(())
	}

	#[test]
	fn test_svg_arc_semicircle() -> Result<()> {
		// -- Setup & Fixtures
		let start = Point::new(100.0, 100.0);
		let end = Point::new(200.0, 100.0);

		// -- Exec
		let segs = arc_to_cubic_beziers(start, 50.0, 50.0, 0.0, false, true, end);

		// -- Check
		assert_eq!(segs.len(), 2);
		if let NormalizedSegment::CubicTo { p, .. } = segs[1] {
			assert!((p.x - 200.0).abs() < 1e-6);
			assert!((p.y - 100.0).abs() < 1e-6);
		} else {
			return Err("Expected cubic bezier segment".into());
		}
		Ok(())
	}
}

// endregion: --- Tests
