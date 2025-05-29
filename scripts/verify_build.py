#!/usr/bin/env python3
"""
Verify that the mopaq project builds correctly.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import subprocess
import sys
import os
from pathlib import Path


def run_command(cmd, cwd=None):
    """Run a command and return success status."""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        return False

    if result.stdout:
        print(result.stdout)

    return True


def main():
    """Verify the build process."""
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)

    print("mopaq Build Verification")
    print("=" * 50)

    # Check Rust installation
    print("\n1. Checking Rust installation...")
    if not run_command(["rustc", "--version"]):
        print("Error: Rust is not installed")
        return 1

    # Clean previous builds
    print("\n2. Cleaning previous builds...")
    run_command(["cargo", "clean"])

    # Check formatting
    print("\n3. Checking code formatting...")
    if not run_command(["cargo", "fmt", "--all", "--check"]):
        print("Error: Code is not properly formatted. Run 'cargo fmt --all'")
        return 1

    # Build all crates
    print("\n4. Building all crates...")
    if not run_command(["cargo", "build", "--all"]):
        print("Error: Build failed")
        return 1

    # Run clippy
    print("\n5. Running clippy...")
    if not run_command(["cargo", "clippy", "--all-features", "--all-targets"]):
        print("Warning: Clippy found issues")

    # Run tests
    print("\n6. Running tests...")
    if not run_command(["cargo", "test", "--all"]):
        print("Error: Tests failed")
        return 1

    # Check documentation
    print("\n7. Checking documentation...")
    if not run_command(["cargo", "doc", "--all-features", "--no-deps"]):
        print("Error: Documentation generation failed")
        return 1

    # Verify FFI header generation
    print("\n8. Verifying FFI header generation...")
    header_path = project_root / "storm-ffi" / "include" / "StormLib.h"
    if header_path.exists():
        print(f"✓ FFI header generated at: {header_path}")
    else:
        print("Warning: FFI header not found")

    # Build release version
    print("\n9. Building release version...")
    if not run_command(["cargo", "build", "--all", "--release"]):
        print("Error: Release build failed")
        return 1

    print("\n" + "=" * 50)
    print("✓ Build verification completed successfully!")

    return 0


if __name__ == "__main__":
    sys.exit(main())
