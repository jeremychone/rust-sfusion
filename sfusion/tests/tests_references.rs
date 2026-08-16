mod _tsupport;

use _tsupport::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_ref_01() -> Result<()> {
	// -- Setup & Fixtures
	let svg_content = include_str!("data/references/ref-01.svg");
	let expected_fusion = include_str!("data/references/ref-01.txt");

	// -- Exec
	let actual_fusion = sfusion::svg_to_sfusion(svg_content)?;

	// -- Check
	assert_fusion_eq(&actual_fusion, expected_fusion, 0.001)?;

	Ok(())
}
