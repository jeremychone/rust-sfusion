use std::collections::HashMap;

// region:    --- Types

#[derive(Debug, Default)]
pub struct NameTracker {
	used_counts: HashMap<String, u32>,
}

// endregion: --- Types

// region:    --- Functions

pub fn sanitize_identifier(raw: &str) -> String {
	let trimmed = raw.trim();
	if trimmed.is_empty() {
		return String::new();
	}

	let mut sanitized: String = trimmed
		.chars()
		.map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
		.collect();

	if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
		sanitized.insert(0, '_');
	}

	sanitized
}

// endregion: --- Functions

// region:    --- Constructors

impl NameTracker {
	pub fn generate_unique_name(&mut self, explicit_id: Option<&str>) -> String {
		let sanitized = explicit_id.map(sanitize_identifier).filter(|s| !s.is_empty());
		match sanitized {
			Some(base) => {
				let count = self.used_counts.entry(base.clone()).or_insert(0);
				if *count == 0 {
					*count += 1;
					base
				} else {
					let assigned_name = format!("{base}_{count}");
					*count += 1;
					assigned_name
				}
			}
			_ => {
				let count = self.used_counts.entry("poly".to_string()).or_insert(0);
				*count += 1;
				format!("poly_{count}")
			}
		}
	}
}

// endregion: --- Constructors

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_fusion_naming_fallback_polys() -> Result<()> {
		// -- Setup & Fixtures
		let mut tracker = NameTracker::default();

		// -- Exec
		let name1 = tracker.generate_unique_name(None);
		let name2 = tracker.generate_unique_name(Some(""));
		let name3 = tracker.generate_unique_name(None);

		// -- Check
		assert_eq!(name1, "poly_1");
		assert_eq!(name2, "poly_2");
		assert_eq!(name3, "poly_3");

		Ok(())
	}

	#[test]
	fn test_fusion_naming_explicit_and_duplicates() -> Result<()> {
		// -- Setup & Fixtures
		let mut tracker = NameTracker::default();

		// -- Exec
		let grabber1 = tracker.generate_unique_name(Some("grabber"));
		let grabber2 = tracker.generate_unique_name(Some("grabber"));
		let grabber3 = tracker.generate_unique_name(Some("grabber"));
		let poly_name = tracker.generate_unique_name(None);

		// -- Check
		assert_eq!(grabber1, "grabber");
		assert_eq!(grabber2, "grabber_1");
		assert_eq!(grabber3, "grabber_2");
		assert_eq!(poly_name, "poly_1");

		Ok(())
	}

	#[test]
	fn test_fusion_naming_sanitization_characters() -> Result<()> {
		// -- Setup & Fixtures
		let mut tracker = NameTracker::default();

		// -- Exec
		let name_hyphen = tracker.generate_unique_name(Some("layer-1-box"));
		let name_space = tracker.generate_unique_name(Some("main title"));
		let name_special = tracker.generate_unique_name(Some("icon#2@home!"));

		// -- Check
		assert_eq!(name_hyphen, "layer_1_box");
		assert_eq!(name_space, "main_title");
		assert_eq!(name_special, "icon_2_home_");

		Ok(())
	}

	#[test]
	fn test_fusion_naming_sanitization_leading_digits() -> Result<()> {
		// -- Setup & Fixtures
		let mut tracker = NameTracker::default();

		// -- Exec
		let name_digit = tracker.generate_unique_name(Some("3d_box"));
		let name_digit_hyphen = tracker.generate_unique_name(Some("123-abc"));

		// -- Check
		assert_eq!(name_digit, "_3d_box");
		assert_eq!(name_digit_hyphen, "_123_abc");

		Ok(())
	}

	#[test]
	fn test_fusion_naming_sanitization_collision() -> Result<()> {
		// -- Setup & Fixtures
		let mut tracker = NameTracker::default();

		// -- Exec
		let name1 = tracker.generate_unique_name(Some("layer-1"));
		let name2 = tracker.generate_unique_name(Some("layer_1"));

		// -- Check
		assert_eq!(name1, "layer_1");
		assert_eq!(name2, "layer_1_1");

		Ok(())
	}
}

// endregion: --- Tests
