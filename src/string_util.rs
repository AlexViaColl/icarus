use std::ffi::CStr;

/// # Safety
///
/// This function should be called with ptr always pointing to a valid null-terminated C-string.
pub unsafe fn cstr_to_string<T>(ptr: *const T) -> String {
    CStr::from_ptr(ptr as *const i8).to_string_lossy().into_owned()
}

pub fn format_uuid(arr: [u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        arr[0],
        arr[1],
        arr[2],
        arr[3],
        arr[4],
        arr[5],
        arr[6],
        arr[7],
        arr[8],
        arr[9],
        arr[10],
        arr[11],
        arr[12],
        arr[13],
        arr[14],
        arr[15]
    )
}
