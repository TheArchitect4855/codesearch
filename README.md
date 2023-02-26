# About
codesearch is a simple command-line tool to conduct ngram-based searches
on your codebase.

# Contributing
Pull requests are always welcome to introduce new features/changes to
codesearch.

## Bug Reports
Please open an issue on GitHub describing the problem. Please include
your platform and any other potentially useful information (too much
info is better than not enough!).

# Installation
## Binaries
Currently, precompiled binaries are unavailable as codesearch is in a
very early state and cross-compilation is finicky.

## Compile from Source
**Requirements:**
- Cargo

**Procedure:**
1. Clone or download this repository
1. Navigate to the repository's directory
1. Run `cargo build --release`. This will output the binary to
`target/release/codesearch`.
1. Setup the binary to work from the command line

	On Unix:
	- Copy the binary to `/bin/`, or
	- Create a smylink from `/bin/codesearch` to the target directory

	On Windows:
	- Copy the binary to your preferred destination
	- Add the binary to `PATH`

# Usage
`codesearch [search term]`

This will search the current working directory. If an index does not exist for this directory, one will be created in `[YOUR HOME DIRECTORY]/.thearchitect/codesearch`.
