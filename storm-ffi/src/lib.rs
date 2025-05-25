//! StormLib-compatible C API for the storm MPQ archive library

use libc::{c_char, c_void};
use std::ptr;

/// Archive handle type
pub type HANDLE = *mut c_void;

/// Invalid handle value
pub const INVALID_HANDLE_VALUE: HANDLE = ptr::null_mut();

/// Error codes
#[repr(C)]
pub enum ErrorCode {
    Success = 0,
    FileNotFound = 2,
    AccessDenied = 5,
    InvalidParameter = 87,
    InsufficientBuffer = 122,
    // Add more error codes as needed
}

/// Open an MPQ archive
#[no_mangle]
pub extern "C" fn SFileOpenArchive(
    _filename: *const c_char,
    _priority: u32,
    _flags: u32,
    _handle: *mut HANDLE,
) -> bool {
    // TODO: Implement
    false
}

/// Close an MPQ archive
#[no_mangle]
pub extern "C" fn SFileCloseArchive(_handle: HANDLE) -> bool {
    // TODO: Implement
    false
}

/// Open a file in the archive
#[no_mangle]
pub extern "C" fn SFileOpenFileEx(
    _archive: HANDLE,
    _filename: *const c_char,
    _search_scope: u32,
    _file_handle: *mut HANDLE,
) -> bool {
    // TODO: Implement
    false
}

/// Close a file
#[no_mangle]
pub extern "C" fn SFileCloseFile(_file: HANDLE) -> bool {
    // TODO: Implement
    false
}

/// Read from a file
#[no_mangle]
pub extern "C" fn SFileReadFile(
    _file: HANDLE,
    _buffer: *mut c_void,
    _to_read: u32,
    _read: *mut u32,
    _overlapped: *mut c_void,
) -> bool {
    // TODO: Implement
    false
}

/// Get file size
#[no_mangle]
pub extern "C" fn SFileGetFileSize(_file: HANDLE, _high: *mut u32) -> u32 {
    // TODO: Implement
    0
}

/// Get last error
#[no_mangle]
pub extern "C" fn GetLastError() -> u32 {
    // TODO: Implement
    0
}

/// Set last error
#[no_mangle]
pub extern "C" fn SetLastError(_error: u32) {
    // TODO: Implement
}
