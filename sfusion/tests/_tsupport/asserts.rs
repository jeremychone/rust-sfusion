type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

/// Asserts that two strings match line by line after trimming leading and trailing whitespace from each line.
///
/// When the content does not match, returns an error formatted in Markdown with the expected and actual blocks.
pub fn assert_lines_trimmed_eq(actual: &str, expected: &str) -> Result<()> {
	let actual_lines: Vec<&str> = actual.lines().map(|l| l.trim()).collect();
	let expected_lines: Vec<&str> = expected.lines().map(|l| l.trim()).collect();

	if actual_lines != expected_lines {
		let message = format!(
			"Did not match\n\nExpected: \n\n```\n{}\n```\n\nActual:\n\n```\n{}\n```\n",
			expected.trim(),
			actual.trim()
		);
		return Err(message.into());
	}

	Ok(())
}

/// Compares two Fusion Lua scripts structurally, matching string tokens exactly and numbers within tolerance.
pub fn assert_fusion_eq(actual: &str, expected: &str, tolerance: f64) -> Result<()> {
	let actual_tokens = tokenize_fusion(actual);
	let expected_tokens = tokenize_fusion(expected);

	if actual_tokens.len() != expected_tokens.len() {
		return Err(format!(
			"Token count mismatch: actual has {} tokens, expected has {}",
			actual_tokens.len(),
			expected_tokens.len()
		)
		.into());
	}

	for (idx, (act, exp)) in actual_tokens.iter().zip(expected_tokens.iter()).enumerate() {
		let act_num = act.parse::<f64>();
		let exp_num = exp.parse::<f64>();

		match (act_num, exp_num) {
			(Ok(a), Ok(e)) => {
				if (a - e).abs() > tolerance {
					return Err(format!(
						"Numerical mismatch at token index {idx}: actual '{act}' ({a}) vs expected '{exp}' ({e}), diff {}",
						(a - e).abs()
					)
					.into());
				}
			}
			_ => {
				if act != exp {
					return Err(format!(
						"Token mismatch at token index {idx}: actual '{act}' vs expected '{exp}'"
					)
					.into());
				}
			}
		}
	}

	Ok(())
}

// region:    --- Support

fn tokenize_fusion(content: &str) -> Vec<String> {
	let mut tokens = Vec::new();
	let mut current = String::new();

	for ch in content.chars() {
		if ch.is_whitespace() || ch == ',' || ch == '=' || ch == '{' || ch == '}' {
			if !current.is_empty() {
				tokens.push(current.clone());
				current.clear();
			}
			if ch == '=' || ch == '{' || ch == '}' {
				tokens.push(ch.to_string());
			}
		} else {
			current.push(ch);
		}
	}

	if !current.is_empty() {
		tokens.push(current);
	}

	tokens
}

// endregion: --- Support
