# sfusion

Rust library to parse SVG content and generate DaVinci Resolve Fusion `Tools = ordered() { ... }` node graphs.

## Usage

Add `sfusion` to `Cargo.toml`:

```toml
[dependencies]
sfusion = { path = "../sfusion" }
```

### Quick Example

```rust
use sfusion::svg_to_sfusion;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_xml = r#"<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40" fill="red"/></svg>"#;
    let fusion_nodes = svg_to_sfusion(svg_xml)?;

    println!("{fusion_nodes}");
    Ok(())
}
```

### Low-Level API

```rust
use sfusion::{build_fusion_doc, parse_svg, serialize_fusion_doc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_doc = parse_svg(svg_xml)?;
    let fusion_doc = build_fusion_doc(&svg_doc)?;
    let fusion_text = serialize_fusion_doc(&fusion_doc);

    println!("{fusion_text}");
    Ok(())
}
```
