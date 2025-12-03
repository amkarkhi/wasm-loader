#![no_std]

use core::panic::PanicInfo;
use core::slice;
use core::str;
use heapless::String;

#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
}

fn log_message(message: &str) {
    unsafe {
        log(message.as_ptr(), message.len());
    }
}

#[no_mangle]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize, _: *const u8, _: usize) -> i32 {
    log_message("[Counter] Starting character count");
    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => {
            log_message("[Counter] ERROR - Invalid UTF-8 input");
            return -1;
        }
    };

    let total_chars = input_str.chars().count();
    let letters = input_str.chars().filter(|c| c.is_alphabetic()).count();
    let digits = input_str.chars().filter(|c| c.is_numeric()).count();
    let spaces = input_str.chars().filter(|c| c.is_whitespace()).count();
    log_message("[Counter] Analysis complete");
    let mut output: String<256> = String::new();
    let _ = output.push_str("Total: ");
    append_number(&mut output, total_chars);
    let _ = output.push_str(" | Letters: ");
    append_number(&mut output, letters);
    let _ = output.push_str(" | Digits: ");
    append_number(&mut output, digits);
    let _ = output.push_str(" | Spaces: ");
    append_number(&mut output, spaces);
    log_message("[Counter] Result = ");
    log_message(output.as_str());
    0
}

fn append_number(s: &mut String<256>, n: usize) {
    let mut buffer = [0u8; 20];
    let mut i = 0;
    let mut num = n;
    if num == 0 {
        let _ = s.push('0');
        return;
    }
    while num > 0 {
        buffer[i] = (num % 10) as u8 + b'0';
        num /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        let _ = s.push(buffer[i] as char);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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
