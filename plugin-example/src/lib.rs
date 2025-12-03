#![no_std]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::{slice, str};
use heapless::String;

/*
Host function: log a message to the host's stdout

This is imported from the host and allows the plugin to send log messages
The host implements this function in the "host" module
*/
#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
}

/// Helper function to safely log messages from the plugin
fn log_message(message: &str) {
    unsafe {
        log(message.as_ptr(), message.len());
    }
}

/// Main plugin entry point
#[no_mangle]
pub extern "C" fn process(
    input_ptr: *const u8,
    input_len: usize,
    env_ptr: *const u8,
    env_len: usize,
) -> i32 {
    // Safety: The host guarantees this memory is valid
    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };
    // Convert to string
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => {
            log_message("[Plugin]: ERROR - Invalid UTF-8 input");
            return -1;
        }
    };
    log_message("[Plugin]: Input received successfully");
    let env_slice = unsafe { slice::from_raw_parts(env_ptr, env_len) };
    let env = match str::from_utf8(env_slice) {
        Ok(s) => s,
        Err(_) => {
            log_message("[Plugin] ERROR - Failed to parse env");
            return -1;
        }
    };
    log_message("[Plugin] Env received: ");
    // Reverse the string
    let mut s = String::<256>::new();
    let reversed: String<64> = input_str.chars().rev().collect();
    // Log the result
    write!(&mut s, "[Plugin]: {} | {}", env, reversed).unwrap();
    log_message("[Plugin]: String reversed successfully");
    log_message(s.as_str());
    log_message("[Plugin]: Result = ");
    log_message(reversed.as_str());

    // Return success code
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log_message("[Env-Reader] PANIC occurred!");
    loop {}
}

// We need a global allocator stub for no_std
// This plugin doesn't use dynamic allocation
#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}
