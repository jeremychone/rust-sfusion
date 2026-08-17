// region:    --- Modules

pub mod ast;
mod error;
pub mod fusion;
pub mod svg;

pub use ast::*;
pub use error::{Error, Result};
pub use fusion::*;
pub use svg::*;

// endregion: --- Modules

// region:    --- Public Functions

/// Converts an SVG XML string into DaVinci Resolve Fusion `Tools = ordered() { ... }` format.
pub fn svg_to_sfusion(svg_content: &str) -> Result<String> {
	svg_to_sfusion_with_options(svg_content, &FusionOptions::default())
}

/// Converts an SVG XML string into DaVinci Resolve Fusion `Tools = ordered() { ... }` format with custom options.
pub fn svg_to_sfusion_with_options(svg_content: &str, options: &FusionOptions) -> Result<String> {
	let svg_doc = svg::parse_svg(svg_content)?;
	let fusion_doc = fusion::build_fusion_doc_with_options(&svg_doc, options)?;
	let fusion_str = fusion::serialize_fusion_doc(&fusion_doc);
	Ok(fusion_str)
}

// endregion: --- Public Functions
