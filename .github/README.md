# GitHub Workflows for mopaq library

This directory contains GitHub Actions workflows for automating various development tasks.

## Available Workflows

### Pull Request (`pull-request.yml`)

Runs when a pull request is opened or updated against the main branch.

**Features:**

- Builds and tests the library on Ubuntu with both stable Rust and MSRV (1.85.0)
- Validates code formatting with `rustfmt`
- Checks for issues with `clippy`
- Runs tests including doc tests
- Cross-platform testing on Windows and macOS
- Runs benchmarks and uploads results as artifacts

### Main Branch CI (`main.yml`)

Runs when changes are pushed to the main branch.

**Features:**

- Builds and tests the library on Ubuntu
- Validates code formatting and checks with `clippy`
- Generates and uploads documentation
- Cross-platform testing on Windows and macOS
- Generates code coverage information and uploads to Codecov

### Release (`release.yml`)

Runs when a tag starting with 'v' is pushed.

**Features:**

- Builds and tests the library in release mode
- Publishes the crate to crates.io if tests pass
- Creates a GitHub release with an automatically generated changelog
- Publishes documentation to GitHub Pages

## Secrets Required

The following secrets need to be configured in your GitHub repository settings:

- `CRATES_IO_TOKEN`: Your API token for crates.io (for publishing)

## How to Use

These workflows run automatically based on the triggers defined in each file. No manual action is needed.

To create a new release:

1. Update the version in `Cargo.toml`
2. Commit the change: `git commit -am "Bump version to x.y.z"`
3. Create and push a new tag: `git tag -a vx.y.z -m "Version x.y.z"` and `git push origin vx.y.z`

The release workflow will automatically build, test, publish to crates.io, and create a GitHub release.
