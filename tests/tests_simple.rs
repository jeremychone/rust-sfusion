type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_simple_init() -> Result<()> {
	// -- Setup & Fixtures

	// -- Exec

	// -- Check
	// assert!(true);

	Ok(())
}
