# sfusion-cli

Command line interface to convert SVG files into DaVinci Resolve Fusion format.

## Installation

```sh
cargo install --path sfusion-cli
```

## Usage

### Convert File to Shape Output

Generates `<stem>_fusion-shape.txt` alongside the source file:

```sh
sfusion to-shape icon.svg
```

### Convert Clipboard Content

Inspects clipboard for SVG content, converts it, and writes the Fusion node data back to the clipboard:

```sh
sfusion clip-swap
```

### General Conversion

Convert via positional path or options:

```sh
# Positional syntax
sfusion input.svg

# Explicit input and output flags
sfusion -i input.svg -o output.txt

# Using the convert subcommand
sfusion convert -i input.svg -o output.txt

# Stdin and stdout pipeline
cat input.svg | sfusion > output.txt
```
