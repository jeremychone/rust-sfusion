# sfusion-cli

Command line interface to convert SVG files into DaVinci Resolve Fusion format.

## Installation

```sh
cargo install --path sfusion-cli
```

## Usage

### Convert File to Shape Output

```sh
# Convert File to Shape Output
sfusion to-shape path/to/icon.svg

# Convert File to Shape Output and append terminal sTransform node
sfusion to-shape path/to/icon.svg --sxf
# Generate `path/to/icon.svg.fusion-shape.txt` alongside the source file:

# Convert Clipboard Content
sfusion clip-swap

# Convert Clipboard Content and append terminal sTransform node
sfusion clip-swap --sxf
# Inspects clipboard for SVG content, converts it, and writes the Fusion node data back to the clipboard:

```

### Convert from Stdin

Pipes SVG content from standard input and outputs DaVinci Resolve Fusion node data to standard output:

```sh
cat input.svg | sfusion > output.txt
cat input.svg | sfusion --sxf > output.txt
```
