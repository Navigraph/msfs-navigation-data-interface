//! The ABI of the standalone build, used by the `StandaloneTransport` in the TS package.
//!
//! The host drives the module the same way the sim drives the gauge:
//! - `navigraph_call_function` is the equivalent of a `NAVIGRAPH_CallFunction` commbus call
//! - `navigraph_update` is the equivalent of a sim frame
//! - Results and events are passed back through the imported `navigraph.send_message`, with the same channels and payloads as the commbus

use std::cell::RefCell;

use crate::interface::FunctionQueue;

thread_local! {
    static FUNCTION_QUEUE: RefCell<FunctionQueue> = RefCell::new(FunctionQueue::default());
}

/// Allocate a buffer of `len` bytes for the host to write into
#[no_mangle]
pub extern "C" fn navigraph_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// Free a buffer allocated with `navigraph_alloc` which was not passed to `navigraph_call_function`
///
/// # Safety
/// `ptr` and `len` must come from a single call to `navigraph_alloc`
#[no_mangle]
pub unsafe extern "C" fn navigraph_dealloc(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len));
}

/// Queue a function call. The payload has the same format as a `NAVIGRAPH_CallFunction` commbus call. Takes ownership of the buffer.
///
/// # Safety
/// `ptr` and `len` must come from a single call to `navigraph_alloc`, with all `len` bytes written
#[no_mangle]
pub unsafe extern "C" fn navigraph_call_function(ptr: *mut u8, len: usize) {
    let payload = Vec::from_raw_parts(ptr, len, len);

    let Ok(args) = String::from_utf8(payload) else {
        sentry::capture_message(
            "NAVIGRAPH_CallFunction payload is not valid UTF-8",
            sentry::Level::Warning,
        );
        return;
    };

    FUNCTION_QUEUE.with(|queue| match queue.try_borrow_mut() {
        Ok(mut queue) => queue.push(&args),
        Err(_) => {
            sentry::capture_message(
                "Unable to borrow processing queue",
                sentry::Level::Warning,
            );
        }
    });
}

/// Run queued functions and send the heartbeat when due. The host should call this once per frame.
#[no_mangle]
pub extern "C" fn navigraph_update() {
    FUNCTION_QUEUE.with(|queue| {
        let Ok(mut queue) = queue.try_borrow_mut() else {
            return;
        };

        if let Err(e) = queue.update() {
            println!("[NAVIGRAPH]: Error encountered in update: {e}");
        }
    });
}
