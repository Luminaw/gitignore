# gitignore

A simple CLI tool to manage and generate `.gitignore` files using templates from [github/gitignore](https://github.com/github/gitignore).

## Features

- **Embedded Templates**: Over 150+ standard gitignore templates included in the binary.
- **Custom Templates**: Easily create and store your own templates in `~/.config/gitignore/`.
- **Smart Appending**: Automatically appends to existing `.gitignore` files while avoiding duplicates.
- **Cross-Platform**: Works on Windows, Linux, and macOS.

## Usage

### Add a Template
Add a template to your current directory's `.gitignore`:
```bash
gitignore add Rust
```
*If a `.gitignore` already exists, it will append the content unless it's identical.*

### List Templates
List all available embedded and custom templates:
```bash
gitignore list
```

### Create Custom Template
Save a local file as a custom template for future use:
```bash
gitignore create my-custom.gitignore
```
*Custom templates are stored in `~/.config/gitignore/`.*

## Installation

### From Source
```bash
git clone --recursive https://github.com/yourusername/gitignore.git
cd gitignore
cargo install --path .
```

## Automation
This repository includes GitHub Actions for:
- **CI**: Build and test on every push to `master`.
- **Releases**: Automatic binary builds and GitHub Releases on tags (e.g., `v1.0.0`).
- **Updates**: Weekly updates to the embedded `ignores` submodule to ensure templates are always current.
