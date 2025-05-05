# mopaq library

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
![Rust Version](https://img.shields.io/badge/rust-1.86%2B-orange.svg)

A Rust library for reading and writing World of Warcraft MPQ archives, designed to be approachable, well-documented, and community-driven.

## ✨ What is this?

MPQ (Mo'PQ or Mike O'Brien Pack) is the archive format used by Blizzard Entertainment's games including World of Warcraft, Diablo, and StarCraft. This library allows you to:

- Read existing MPQ archives
- Create new MPQ archives
- Extract files from archives
- Add files to archives
- And more!

## 🚀 Quick Start

### Installation

Add the library to your Cargo.toml:

```toml
[dependencies]
mopaq = "0.1.0"
```

### Basic Usage

```rust
use mopaq::{MpqArchive, Result};
use std::path::Path;

fn main() -> Result<()> {
    // Open an existing WoW MPQ archive
    let archive = MpqArchive::open("path/to/your/wow.mpq")?;
    println!("Archive opened successfully!");

    // Print some basic information
    println!("Format version: {}", archive.header.format_version);
    println!("Contains user header: {}", archive.user_header.is_some());

    // Create a new MPQ archive (format version 1)
    let new_archive = MpqArchive::create("path/to/new.mpq", 1)?;
    println!("Created a new archive!");

    Ok(())
}
```

## 🧰 Features

- **Simple API**: Designed to be intuitive for Rust beginners
- **Well-documented**: Every function has clear explanations
- **Safe**: Strong emphasis on proper error handling
- **Fast**: Optimized for performance with benchmarks
- **Cross-platform**: Works on Windows, macOS, and Linux

## 🛠️ For Beginners

New to Rust or MPQ archives? No problem! We've designed this library to be approachable:

1. **Comprehensive examples** in the `examples/` directory
2. **Step-by-step tutorials** in the documentation
3. **Clear error messages** that help you understand what went wrong

### Learning Resources

- [What are MPQ archives?](#mpq-basics) - An introduction for beginners
- [Rust basics for this project](#rust-basics) - Key Rust concepts used here
- [Common tasks guide](#common-tasks) - How to perform typical operations

## 🤝 Contributing

We welcome contributions from everyone, regardless of experience level! Here's how you can help:

- **Beginners**: Try using the library and report any confusing parts
- **Documentation**: Help improve explanations or add examples
- **Code**: Implement missing features or optimize existing ones
- **Testing**: Add test cases or find bugs

Check out [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

### Getting Started as a Contributor

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/mopaq.git`
3. Create a branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Push your branch: `git push origin my-feature`
7. Open a Pull Request

## 📚 Documentation

Full documentation is available at [docs.rs/mopaq](https://docs.rs/mopaq).

### <a name="mpq-basics"></a>What are MPQ Archives?

MPQ (Mo'PQ or Mike O'Brien Pack) is an archive format created by Blizzard Entertainment. It's used in many of their games to package game assets like models, textures, sounds, and more.

Key features of the MPQ format:

- File compression
- Built-in encryption
- Support for multiple languages
- Fast file lookup through hash tables

### <a name="rust-basics"></a>Rust Basics for This Project

This library makes use of several Rust concepts:

- **Result/Option types**: For proper error handling
- **Traits**: For consistent interfaces
- **Ownership**: For memory safety without garbage collection
- **Modules**: For code organization

Don't worry if you're not familiar with all of these—our documentation explains the concepts as they come up!

### <a name="common-tasks"></a>Common Tasks

#### Opening an Archive

```rust
let archive = MpqArchive::open("game.mpq")?;
```

#### Creating a New Archive

```rust
let archive = MpqArchive::create("new.mpq", 1)?;
```

#### Extracting a File

```rust
// Coming soon!
```

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## 📊 Project Status

This project is under active development. Check our [TODO.md](TODO.md) for planned features.

## 💬 Community

- **Discord**: [Join our server](https://discord.gg/your-invite-link)
- **Issues**: [GitHub Issues](https://github.com/your-username/mopaq/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/mopaq/discussions)

We're building a friendly community around this project and welcome everyone from beginners to experts!
