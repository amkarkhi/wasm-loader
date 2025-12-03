//! ROT13 Cipher Plugin
//!
//! Applies ROT13 cipher to the input (shifts letters by 13 positions)

#![no_std]

use core::panic::PanicInfo;
use core::slice;
use core::str;

#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
}

fn log_message(message: &str) {
    unsafe {
        log(message.as_ptr(), message.len());
    }
}

fn rot13_char(c: char) -> char {
    match c {
        'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
        'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
        _ => c,
    }
}

#[no_mangle]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize, _: *const u8, _: usize) -> i32 {
    log_message("[ROT13] Starting cipher");

    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };

    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => {
            log_message("[ROT13] ERROR - Invalid UTF-8 input");
            return -1;
        }
    };

    log_message("[ROT13] Applying ROT13 cipher");

    // Apply ROT13
    let encoded: heapless::String<256> = input_str.chars().map(rot13_char).collect();

    log_message("[ROT13] Result = ");
    log_message(encoded.as_str());

    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log_message("[ROT13] PANIC occurred!");
    loop {}
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
