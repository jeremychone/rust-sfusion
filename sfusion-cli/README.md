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
sfusion to-shape pat/to/icon.svg
# Generate `path/to/icon.svg.fusion-shape.txt` alongside the source file:

# Convert Clipboard Content
sfusion clip-swap
# Inspects clipboard for SVG content, converts it, and writes the Fusion node data back to the clipboard:

```

### Convert from Stdin

Pipes SVG content from standard input and outputs DaVinci Resolve Fusion node data to standard output:

```sh
cat input.svg | sfusion > output.txt
```
