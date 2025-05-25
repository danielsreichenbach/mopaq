#!/usr/bin/env python3
"""
Test that the CLI binary is correctly named storm-cli.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import subprocess
import sys
import os
from pathlib import Path


def check_binary_name():
    """Check that the binary is named storm-cli."""
    project_root = Path(__file__).parent.parent

    # Build the CLI
    print("Building storm-cli...")
    result = subprocess.run(
        ["cargo", "build", "-p", "storm-cli"],
        cwd=project_root,
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Build failed: {result.stderr}")
        return False

    # Check debug binary
    debug_binary = project_root / "target" / "debug" / "storm-cli"
    if sys.platform == "win32":
        debug_binary = debug_binary.with_suffix(".exe")

    if debug_binary.exists():
        print(f"✓ Debug binary found: {debug_binary}")
    else:
        print(f"✗ Debug binary not found at: {debug_binary}")
        return False

    # Check that storm binary doesn't exist (old name)
    old_binary = project_root / "target" / "debug" / "storm"
    if sys.platform == "win32":
        old_binary = old_binary.with_suffix(".exe")

    if old_binary.exists():
        print(f"⚠ Warning: Old binary name still exists: {old_binary}")
        print("  This might cause confusion. Consider running 'cargo clean'.")

    # Try running the binary
    print("\nTesting binary execution...")
    result = subprocess.run(
        [str(debug_binary), "--version"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        print(f"✓ Binary executes successfully")
        print(f"  Version: {result.stdout.strip()}")
        return True
    else:
        print(f"✗ Binary failed to execute: {result.stderr}")
        return False


def main():
    """Main entry point."""
    print("Testing storm-cli binary name...")
    print("-" * 40)

    if check_binary_name():
        print("\n✓ All checks passed!")
        print("\nThe CLI binary is correctly named 'storm-cli'")
        return 0
    else:
        print("\n✗ Some checks failed!")
        return 1


if __name__ == "__main__":
    sys.exit(main())
