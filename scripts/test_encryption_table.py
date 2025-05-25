#!/usr/bin/env python3
"""
Test and visualize the MPQ encryption table generation.

This script implements the same algorithm in Python to verify
the Rust implementation is correct.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

def generate_encryption_table():
    """Generate the MPQ encryption table."""
    table = [0] * 0x500
    seed = 0x00100001

    for index1 in range(0x100):
        for index2 in range(5):
            table_index = index1 + index2 * 0x100

            # Update seed
            seed = (seed * 125 + 3) % 0x2AAAAB
            temp1 = (seed & 0xFFFF) << 0x10

            seed = (seed * 125 + 3) % 0x2AAAAB
            temp2 = seed & 0xFFFF

            table[table_index] = temp1 | temp2

    return table


def print_table_section(table, start, end, name):
    """Print a section of the table."""
    print(f"\n{name} (indices 0x{start:03X} - 0x{end-1:03X}):")
    for i in range(start, end):
        if i % 4 == 0:
            print(f"  [{i:03X}]: ", end="")
        print(f"0x{table[i]:08X}", end="")
        if i % 4 == 3:
            print()
        else:
            print(", ", end="")
    if (end - start) % 4 != 0:
        print()


def verify_known_values(table):
    """Verify known values from the MPQ specification."""
    known_values = {
        0x000: 0x55C636E2,
        0x001: 0x02BE0170,
        0x002: 0x584B71D4,
        0x003: 0x2984F00E,
        0x004: 0xB682C809,
        0x100: 0x76F8C1B1,
        0x200: 0x3DF6965D,
        0x300: 0x15F261D3,
        0x400: 0x193AA698,
        0x4FB: 0x6149809C,
        0x4FC: 0xB0099EF4,
        0x4FD: 0xC5F653A5,
        0x4FE: 0x4C10790D,
        0x4FF: 0x7303286C,
    }

    print("Verifying known values:")
    all_correct = True

    for index, expected in known_values.items():
        actual = table[index]
        if actual == expected:
            print(f"  ✓ [0x{index:03X}]: 0x{actual:08X}")
        else:
            print(f"  ✗ [0x{index:03X}]: got 0x{actual:08X}, expected 0x{expected:08X}")
            all_correct = False

    return all_correct


def generate_rust_test_code(table):
    """Generate Rust test code for additional values."""
    print("\nRust test code for additional values:")
    print("```rust")

    # Generate tests for boundaries of each sub-table
    for subtable in range(5):
        base = subtable * 0x100
        print(f"// Sub-table {subtable} (offset 0x{base:03X})")
        for offset in [0, 1, 0x7F, 0x80, 0xFE, 0xFF]:
            if base + offset < 0x500:
                index = base + offset
                value = table[index]
                print(f"assert_eq!(ENCRYPTION_TABLE[0x{index:03X}], 0x{value:08X});")
        print()

    print("```")


def test_encryption_algorithm():
    """Test the encryption/decryption algorithm."""
    print("\nTesting encryption algorithm:")

    # Original data from MPQ spec
    original = [
        0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0,
        0xFEDCBA98, 0x76543210, 0xF0DEBC9A, 0xE1C3A597
    ]

    key = 0xC1EB1CEF
    table = generate_encryption_table()

    # Encrypt
    data = original.copy()
    seed = 0xEEEEEEEE

    encrypted_data = []
    for i in range(len(data)):
        # Update seed
        seed = (seed + table[0x400 + (key & 0xFF)]) & 0xFFFFFFFF

        # Store original value
        ch = data[i]

        # Encrypt
        encrypted = ch ^ ((key + seed) & 0xFFFFFFFF)
        encrypted_data.append(encrypted)
        data[i] = encrypted

        # Update key
        key = (((~key << 0x15) + 0x11111111) | (key >> 0x0B)) & 0xFFFFFFFF

        # Update seed
        seed = ((ch + seed + (seed << 5) + 3) & 0xFFFFFFFF)

    print("Original data:")
    for i, val in enumerate(original):
        print(f"  [{i}]: 0x{val:08X}")

    print("\nEncrypted data:")
    for i, val in enumerate(encrypted_data):
        print(f"  [{i}]: 0x{val:08X}")

    # Test decryption
    key = 0xC1EB1CEF  # Reset key
    seed = 0xEEEEEEEE
    decrypted = []

    for i in range(len(encrypted_data)):
        # Update seed
        seed = (seed + table[0x400 + (key & 0xFF)]) & 0xFFFFFFFF

        # Decrypt
        ch = encrypted_data[i] ^ ((key + seed) & 0xFFFFFFFF)
        decrypted.append(ch)

        # Update key
        key = (((~key << 0x15) + 0x11111111) | (key >> 0x0B)) & 0xFFFFFFFF

        # Update seed
        seed = ((ch + seed + (seed << 5) + 3) & 0xFFFFFFFF)

    print("\nDecrypted data:")
    all_match = True
    for i, (dec, orig) in enumerate(zip(decrypted, original)):
        match = "✓" if dec == orig else "✗"
        print(f"  [{i}]: 0x{dec:08X} {match}")
        if dec != orig:
            all_match = False

    if all_match:
        print("\n✓ Round-trip encryption/decryption successful!")
    else:
        print("\n✗ Round-trip failed!")


def main():
    """Main entry point."""
    print("MPQ Encryption Table Test")
    print("========================")

    # Generate table
    table = generate_encryption_table()

    # Verify known values
    if verify_known_values(table):
        print("\n✓ All known values are correct!")
    else:
        print("\n✗ Some values don't match!")

    # Print sections of the table
    print_table_section(table, 0x000, 0x008, "Table start")
    print_table_section(table, 0x0FC, 0x104, "Around first boundary")
    print_table_section(table, 0x1FC, 0x204, "Around second boundary")
    print_table_section(table, 0x2FC, 0x304, "Around third boundary")
    print_table_section(table, 0x3FC, 0x404, "Around fourth boundary")
    print_table_section(table, 0x4F8, 0x500, "Table end")

    # Generate test code
    generate_rust_test_code(table)

    # Test encryption
    test_encryption_algorithm()

    print("\n✓ Encryption table test completed!")


if __name__ == "__main__":
    main()
