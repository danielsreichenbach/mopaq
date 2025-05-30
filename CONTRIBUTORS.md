# Contributors

Thank you to everyone who has contributed to the `mopaq` project!

## Project Lead

- **Daniel S. Reichenbach** ([@danielsreichenbach](https://github.com/danielsreichenbach)) - Project creator and maintainer

## Core Contributors

*This section will be updated as the project grows and receives contributions.*

## How to Contribute

We welcome contributions from the community! Here are some ways you can help:

### Code Contributions

1. **Fork the repository** and create your feature branch (`git checkout -b feature/amazing-feature`)
2. **Make your changes** following the Rust style guidelines
3. **Add tests** for any new functionality
4. **Ensure all tests pass** (`cargo test --all-features`)
5. **Run clippy** (`cargo clippy --all-features --all-targets -- -D warnings`)
6. **Format your code** (`cargo fmt --all`)
7. **Commit your changes** with descriptive commit messages
8. **Push to your branch** and open a Pull Request

### Other Ways to Contribute

- **Report bugs**: Open an issue describing the problem
- **Suggest features**: Open an issue with your enhancement proposal
- **Improve documentation**: Help make our docs clearer and more comprehensive
- **Add examples**: Create examples showing different use cases
- **Performance improvements**: Profile and optimize the code

### Areas Where Help is Needed

Based on our TODO.md, here are some areas where contributions would be especially welcome:

1. **Encryption support in ArchiveBuilder**
   - Implement file encryption in `write_file` method
   - Add FIX_KEY flag support
   - Test encrypted file round-trips

2. **Sector CRC support**
   - Generate CRC table for multi-sector files
   - Add CRC generation to ArchiveBuilder
   - Test CRC validation round-trips

3. **Version 4 format support**
   - Implement v4 header writing with MD5 checksums
   - Calculate MD5 for tables (hash, block, hi-block)
   - Add MD5 header validation

### Development Guidelines

- **Code Style**: Follow Rust idioms and conventions
- **Documentation**: Document public APIs and complex implementations
- **Testing**: Write tests for new functionality
- **Performance**: Consider performance implications of changes
- **Compatibility**: Maintain backwards compatibility when possible

### Recognition

All contributors will be recognized in this file. Significant contributions may also be highlighted in release notes.

## License

By contributing to this project, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

## Contact

- Open an issue for questions or discussions
- For security concerns, please see SECURITY.md

---

*Want to see your name here? We'd love to have your contribution!*
