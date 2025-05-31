#!/usr/bin/env python3
"""Generate test data for storm-cli archive create command testing"""

import os
import shutil
import random
import string
import argparse
from pathlib import Path


def generate_random_content(size_kb):
    """Generate random content of approximately the specified size in KB"""
    target_size = size_kb * 1024
    content = []

    while len(''.join(content)) < target_size:
        # Mix different types of content
        choice = random.choice(['text', 'binary', 'structured'])

        if choice == 'text':
            # Generate Lorem ipsum style text
            words = ['lorem', 'ipsum', 'dolor', 'sit', 'amet', 'consectetur',
                    'adipiscing', 'elit', 'sed', 'do', 'eiusmod', 'tempor',
                    'incididunt', 'ut', 'labore', 'et', 'dolore', 'magna']
            paragraph = ' '.join(random.choice(words) for _ in range(random.randint(50, 200)))
            content.append(paragraph + '\n\n')

        elif choice == 'binary':
            # Generate binary-like data
            binary_data = ''.join(random.choice('0123456789ABCDEF') for _ in range(256))
            content.append(binary_data + '\n')

        else:  # structured
            # Generate JSON-like or CSV-like data
            if random.choice([True, False]):
                # JSON-like
                json_data = f'{{"id": {random.randint(1, 1000)}, "value": "{random.choice(string.ascii_letters) * 10}"}}'
                content.append(json_data + '\n')
            else:
                # CSV-like
                csv_data = f'{random.randint(1, 100)},{random.choice(string.ascii_letters)},{random.random():.3f}'
                content.append(csv_data + '\n')

    result = ''.join(content)
    return result[:target_size]  # Trim to exact size


def create_test_directory(base_path, config):
    """Create a directory structure with test files"""
    base_path = Path(base_path)

    # Clean up if exists
    if base_path.exists():
        shutil.rmtree(base_path)

    base_path.mkdir(parents=True)

    # Create directory structure
    directories = config.get('directories', [])
    for dir_path in directories:
        (base_path / dir_path).mkdir(parents=True, exist_ok=True)

    # Create files
    files_created = []
    for file_config in config.get('files', []):
        file_path = base_path / file_config['path']
        file_path.parent.mkdir(parents=True, exist_ok=True)

        # Generate content based on type
        if file_config['type'] == 'text':
            content = generate_random_content(file_config.get('size_kb', 1))
            file_path.write_text(content)
        elif file_config['type'] == 'binary':
            size_bytes = file_config.get('size_kb', 1) * 1024
            content = os.urandom(size_bytes)
            file_path.write_bytes(content)
        elif file_config['type'] == 'empty':
            file_path.touch()

        files_created.append(str(file_path.relative_to(base_path)))

    return files_created


# Test configurations
TEST_CONFIGS = {
    'simple': {
        'description': 'Simple flat structure with text files',
        'files': [
            {'path': 'readme.txt', 'type': 'text', 'size_kb': 2},
            {'path': 'data.txt', 'type': 'text', 'size_kb': 5},
            {'path': 'config.ini', 'type': 'text', 'size_kb': 1},
        ]
    },

    'game_assets': {
        'description': 'Game-like asset structure',
        'directories': ['textures', 'models', 'sounds', 'scripts'],
        'files': [
            {'path': 'textures/player.dds', 'type': 'binary', 'size_kb': 256},
            {'path': 'textures/terrain.dds', 'type': 'binary', 'size_kb': 512},
            {'path': 'models/player.mdx', 'type': 'binary', 'size_kb': 128},
            {'path': 'models/building.mdx', 'type': 'binary', 'size_kb': 64},
            {'path': 'sounds/music/theme.mp3', 'type': 'binary', 'size_kb': 1024},
            {'path': 'sounds/effects/click.wav', 'type': 'binary', 'size_kb': 32},
            {'path': 'scripts/main.lua', 'type': 'text', 'size_kb': 10},
            {'path': 'scripts/utils.lua', 'type': 'text', 'size_kb': 5},
        ]
    },

    'nested': {
        'description': 'Deeply nested directory structure',
        'files': [
            {'path': 'level1/readme.txt', 'type': 'text', 'size_kb': 1},
            {'path': 'level1/level2/data.bin', 'type': 'binary', 'size_kb': 10},
            {'path': 'level1/level2/level3/config.xml', 'type': 'text', 'size_kb': 2},
            {'path': 'level1/level2/level3/level4/deep.txt', 'type': 'text', 'size_kb': 1},
        ]
    },

    'mixed_sizes': {
        'description': 'Mix of file sizes from tiny to large',
        'files': [
            {'path': 'tiny.txt', 'type': 'empty', 'size_kb': 0},
            {'path': 'small.dat', 'type': 'binary', 'size_kb': 1},
            {'path': 'medium.bin', 'type': 'binary', 'size_kb': 100},
            {'path': 'large.pak', 'type': 'binary', 'size_kb': 1024},
            {'path': 'config.json', 'type': 'text', 'size_kb': 5},
        ]
    },

    'special_names': {
        'description': 'Files with special characters and spaces',
        'files': [
            {'path': 'file with spaces.txt', 'type': 'text', 'size_kb': 1},
            {'path': 'special-chars_$#@.dat', 'type': 'binary', 'size_kb': 5},
            {'path': 'unicode_文件.txt', 'type': 'text', 'size_kb': 2},
            {'path': '.hidden_file', 'type': 'text', 'size_kb': 1},
        ]
    }
}


def main():
    parser = argparse.ArgumentParser(description='Generate test data for storm-cli testing')
    parser.add_argument('config', choices=list(TEST_CONFIGS.keys()) + ['all'],
                       help='Test configuration to generate')
    parser.add_argument('--output-dir', default='test-data/raw-data',
                       help='Output directory (default: test-data/raw-data)')

    args = parser.parse_args()

    output_base = Path(args.output_dir)

    configs_to_generate = []
    if args.config == 'all':
        configs_to_generate = list(TEST_CONFIGS.keys())
    else:
        configs_to_generate = [args.config]

    print(f"Generating test data in: {output_base}")
    print()

    for config_name in configs_to_generate:
        config = TEST_CONFIGS[config_name]
        output_dir = output_base / config_name

        print(f"Creating '{config_name}': {config['description']}")
        files = create_test_directory(output_dir, config)
        print(f"  Created {len(files)} files in {output_dir}")
        for file in sorted(files):
            print(f"    - {file}")
        print()

    print("Test data generation complete!")
    print()
    print("Example usage:")
    print(f"  storm archive create test.mpq {output_base}/simple")
    print(f"  storm archive create game.mpq {output_base}/game_assets --compression bzip2")
    print(f"  storm archive create nested.mpq {output_base}/nested --recursive")


if __name__ == '__main__':
    main()
