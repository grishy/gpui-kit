#![cfg(target_os = "macos")]

use cocoa::base::{BOOL, NO, YES, id, nil};
use gpui::Platform;
use gpui_macos::MacPlatform;
use objc::{Encode, class, msg_send, sel, sel_impl};
use std::{cell::Cell, ffi::c_void, rc::Rc};

fn main() {
    // A custom harness keeps AppKit and MacPlatform on the actual main thread.
    let platform = Rc::new(MacPlatform::new(true));
    let selector = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
    let delegate_class = class!(GPUIApplicationDelegate);
    let method = delegate_class.instance_method(selector).unwrap();
    assert_eq!(method.return_type(), BOOL::encode());
    assert_eq!(method.argument_type(3), Some(BOOL::encode()));

    unsafe {
        let delegate: id = msg_send![delegate_class, new];
        (*delegate).set_ivar("platform", Rc::as_ptr(&platform) as *const c_void);

        for visible in [NO, YES] {
            let result: BOOL = msg_send![delegate,
                applicationShouldHandleReopen: nil hasVisibleWindows: visible];
            assert_eq!(result, YES, "AppKit handles reopen without a callback");
        }

        let calls = Rc::new(Cell::new(0));
        let callback_calls = calls.clone();
        let weak_platform = Rc::downgrade(&platform);
        platform.on_reopen(Box::new(move || {
            callback_calls.set(callback_calls.get() + 1);
            // Re-enter the platform lock and replace the callback during dispatch.
            let replacement_calls = callback_calls.clone();
            weak_platform
                .upgrade()
                .unwrap()
                .on_reopen(Box::new(move || {
                    replacement_calls.set(replacement_calls.get() + 10);
                }));
        }));

        for (visible, expected_calls) in [(YES, 1), (NO, 11), (YES, 21)] {
            let result: BOOL = msg_send![delegate,
                applicationShouldHandleReopen: nil hasVisibleWindows: visible];
            assert_eq!(result, NO, "the application callback owns reopen");
            assert_eq!(calls.get(), expected_calls);
        }

        (*delegate).set_ivar("platform", std::ptr::null::<c_void>());
        let _: () = msg_send![delegate, release];
    }
    println!("reopen: native BOOL ABI, both visibility flags, callback replacement passed");
}
