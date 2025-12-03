use core::slice;
use core::str;
use heapless::String;
use wasm_shared::plugin_helpers::*;

#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
}

fn log_message(message: &str) {
    unsafe {
        log(message.as_ptr(), message.len());
    }
}

/// Example plugin demonstrating proper error handling
/// This plugin validates input and demonstrates error codes
#[no_mangle]
pub unsafe extern "C" fn process(
    input_ptr: *const u8,
    input_len: usize,
    _env_ptr: *const u8,
    _env_len: usize,
) -> i32 {
    log_message("[ValidatorPlugin] Starting validation");

    // Step 1: Validate input pointer
    if input_ptr.is_null() {
        log_message("[ValidatorPlugin] ERROR: Null input pointer");
        return ERROR_INVALID_INPUT;
    }

    // Step 2: Read input safely
    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };
    
    // Step 3: Validate UTF-8
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => {
            log_message("[ValidatorPlugin] Input is valid UTF-8");
            s
        }
        Err(_) => {
            log_message("[ValidatorPlugin] ERROR: Invalid UTF-8");
            return ERROR_INVALID_UTF8;
        }
    };

    // Step 4: Check input length
    if input_str.is_empty() {
        log_message("[ValidatorPlugin] ERROR: Empty input");
        return ERROR_INVALID_INPUT;
    }

    if input_str.len() > 1000 {
        log_message("[ValidatorPlugin] ERROR: Input too large");
        return ERROR_BUFFER_OVERFLOW;
    }

    // Step 5: Validate content (only alphanumeric and spaces)
    for ch in input_str.chars() {
        if !ch.is_alphanumeric() && !ch.is_whitespace() {
            log_message("[ValidatorPlugin] ERROR: Invalid characters in input");
            return ERROR_PARSE_ERROR;
        }
    }

    // Step 6: Process successfully
    let mut output: String<256> = String::new();
    let _ = output.push_str("Valid input: ");
    let _ = output.push_str(input_str);
    
    log_message("[ValidatorPlugin] Validation successful!");
    log_message("[ValidatorPlugin] Result = ");
    log_message(output.as_str());

    SUCCESS
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}
