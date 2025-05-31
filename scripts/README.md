# Helper scripts

During development it just so happened that many types of MPQ archives are just
too big, not available, etc.

## `mpq_tool.py`

This little swiss army knife is hopefully letting me implement any kind of MPQ
archive.

```shell
# Create different types of test archives
python3 mpq_tools.py create minimal --version 1
python3 mpq_tools.py create compressed --compression bzip2
python3 mpq_tools.py create comprehensive --all-versions
python3 mpq_tools.py create crc
python3 mpq_tools.py create edge-cases

# Verify implementations
python3 mpq_tools.py verify encryption-table
python3 mpq_tools.py verify hash-function

# Analyze existing archives
python3 mpq_tools.py info archive.mpq
```

The general idea here is to produce any kind of archive on the fly instead of
having to resort to trickery such as Git-LFS, or worse things such as making you
(or me) redownload a few GB of archives in every test run.

To spare us the PITA, we will just generate them.

## `generate_test_data.py`

To verify archive creation, raw files would help. We create a random set of
these using this script.

```shell
python3 scripts/generate_test_data.py all
```

Supported modes are:

- `simple`
- `game_assets`
- `nested`
- `mixed_sizes`
- `special_names`
- `all`

A target directory can be specified using `--output-dir path/to/save/stuff`.
The script will default to using `--output-dir test-data/raw-data`.
