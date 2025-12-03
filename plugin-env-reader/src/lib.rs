#![no_std]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use heapless::String;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Env {
    timestamp: i64,
    random_seed: i64,
}

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
pub extern "C" fn process(_: *const u8, _: usize, env_ptr: *const u8, env_len: usize) -> i32 {
    log_message("[Env-Reader] Starting conversion");
    let env_slice = unsafe { core::slice::from_raw_parts(env_ptr, env_len) };
    // Parse JSON using serde_json_core
    let result: Result<(Env, _), _> = serde_json_core::from_slice(env_slice);
    match result {
        Ok((env, _)) => {
            let mut s = String::<256>::new();
            write!(
                &mut s,
                "timestamp = {} random_seed = {}",
                env.timestamp, env.random_seed
            )
            .unwrap();
            log_message(&s);
            0
        }
        Err(_) => {
            log_message("[Env-Reader] ERROR - Failed to parse JSON");
            -1
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log_message("[Env-Reader] PANIC occurred!");
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
