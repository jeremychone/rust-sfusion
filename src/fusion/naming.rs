use std::collections::HashMap;

// region:    --- Types

#[derive(Debug, Default)]
pub struct NameTracker {
	used_counts: HashMap<String, u32>,
}

// endregion: --- Types

// region:    --- Constructors

impl NameTracker {
	pub fn generate_unique_name(&mut self, explicit_id: Option<&str>) -> String {
		match explicit_id {
			Some(id) if !id.trim().is_empty() => {
				let base = id.trim();
				let count = self.used_counts.entry(base.to_string()).or_insert(0);
				if *count == 0 {
					*count += 1;
					base.to_string()
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
}

// endregion: --- Tests
