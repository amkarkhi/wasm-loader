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

#[no_mangle]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize, _: *const u8, _: usize) -> i32 {
    log_message("[Uppercase] Starting conversion");
    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => {
            log_message("[Uppercase] ERROR - Invalid UTF-8 input");
            return -1;
        }
    };
    log_message("[Uppercase] Converting to uppercase");
    let uppercase: heapless::String<64> =
        input_str.chars().map(|c| c.to_ascii_uppercase()).collect();
    log_message("[Uppercase] Result = ");
    log_message(uppercase.as_str());
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log_message("[Uppercase] PANIC occurred!");
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
