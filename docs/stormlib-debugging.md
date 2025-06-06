# StormLib Debugging Guide

This document describes how to compile and run test programs against the original StormLib implementation for debugging and verification purposes.

## Prerequisites

- StormLib source code available at `/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib`
- GCC compiler
- CMake (for building StormLib if needed)

## Compiling Test Programs Against StormLib

### 1. Basic Compilation

For a simple test program that uses StormLib:

```bash
gcc -o test_program test_program.c \
    -I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src \
    -L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build \
    -lstorm -lz -lbz2
```

### 2. Debug Build with Symbols

For debugging with gdb:

```bash
gcc -g -O0 -o test_program test_program.c \
    -I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src \
    -L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build \
    -lstorm -lz -lbz2
```

### 3. With Additional Debug Output

To enable StormLib's internal debug output:

```bash
gcc -g -O0 -DSTORMLIB_DEBUG -o test_program test_program.c \
    -I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src \
    -L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build \
    -lstorm -lz -lbz2
```

## Example Test Program Structure

```c
#include <stdio.h>
#include <StormLib.h>

int main() {
    HANDLE hMpq = NULL;
    
    // Open MPQ archive
    if (!SFileOpenArchive("path/to/archive.mpq", 0, 0, &hMpq)) {
        printf("Failed to open archive: %d\n", GetLastError());
        return 1;
    }
    
    // Your test code here
    
    // Close archive
    SFileCloseArchive(hMpq);
    return 0;
}
```

## Common Debugging Scenarios

### 1. Checking Table Formats

When debugging table format issues (like HET/BET tables):

```c
// Example: Checking if tables are compressed/encrypted
HANDLE hFile = NULL;
if (SFileOpenFileEx(hMpq, "(hash table)", 0, &hFile)) {
    DWORD dwFileSize = SFileGetFileSize(hFile, NULL);
    BYTE* buffer = malloc(dwFileSize);
    SFileReadFile(hFile, buffer, dwFileSize, &dwBytesRead, NULL);
    
    // Examine first few bytes
    printf("First 16 bytes: ");
    for (int i = 0; i < 16 && i < dwFileSize; i++) {
        printf("%02X ", buffer[i]);
    }
    printf("\n");
    
    free(buffer);
    SFileCloseFile(hFile);
}
```

### 2. Tracing Decompression

To debug compression issues:

```c
// Set breakpoints in StormLib's SCompDecompress function
// Or add debug output to trace compression methods being used
```

### 3. Examining Raw Table Data

For low-level table debugging:

```c
// Read raw table data without StormLib's processing
FILE* fp = fopen("archive.mpq", "rb");
fseek(fp, table_offset, SEEK_SET);
fread(buffer, 1, table_size, fp);
// Examine raw bytes
```

## Building StormLib with Debug Symbols

If you need to step through StormLib code:

```bash
cd /home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib
mkdir build-debug
cd build-debug
cmake -DCMAKE_BUILD_TYPE=Debug ..
make
```

Then link against the debug build:

```bash
gcc -g -O0 -o test_program test_program.c \
    -I../src \
    -L. \
    -lstorm -lz -lbz2
```

## Running with GDB

```bash
gdb ./test_program
(gdb) break main
(gdb) run
(gdb) step
```

## Useful StormLib Internal Functions

For deep debugging, you can access these internal functions:

- `AllocateMpqHeader()` - Header allocation
- `LoadMpqTables()` - Table loading logic
- `AllocateHashTable()` - Hash table allocation
- `DecryptMpqBlock()` - Block decryption
- `SCompDecompress()` - Decompression dispatcher

## Memory Debugging

To check for memory issues:

```bash
valgrind --leak-check=full ./test_program
```

## Example Debug Session from HET/BET Investigation

During the HET/BET debugging, we used this approach:

1. Created `debug_stormlib.c` to verify StormLib could read the archive
2. Created `debug_table_format.c` to examine raw table bytes
3. Created `debug_het_size.c` to analyze table structure sizes
4. Created `debug_stormlib_trace.c` to trace decompression attempts

The key insight came from examining the raw bytes and seeing that the tables were unencrypted but our code was trying to decrypt them.

## Tips

1. Always check StormLib's error codes with `GetLastError()`
2. Use hex dumps to compare byte-by-byte with expected formats
3. Check for endianness issues when reading multi-byte values
4. Remember that StormLib uses Windows-style error codes
5. Some MPQ files have quirks - test with multiple archives

## References

- StormLib source: `/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib`
- MPQ format docs: http://www.zezula.net/en/mpq/mpqformat.html
- Test archives: `/home/danielsreichenbach/Downloads/wow/*/Data/*.MPQ`