#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    use gpui::Platform;
    use gpui_macos::MacPlatform;
    use objc2::{
        Encode, class, msg_send,
        runtime::{AnyObject, Bool},
        sel,
    };
    use std::{cell::Cell, ffi::c_void, rc::Rc};

    // A custom harness keeps AppKit and MacPlatform on the actual main thread.
    let platform = Rc::new(MacPlatform::new(true));
    let selector = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
    let delegate_class = class!(GPUIApplicationDelegate);
    let method = delegate_class.instance_method(selector).unwrap();
    assert!(Bool::ENCODING.equivalent_to_str(method.return_type().to_str().unwrap()));
    assert!(Bool::ENCODING.equivalent_to_str(method.argument_type(3).unwrap().to_str().unwrap()));

    // SAFETY: this main-thread probe owns the delegate, and platform outlives its ivar.
    unsafe {
        let delegate: *mut AnyObject = msg_send![delegate_class, new];
        let ivar = delegate_class.instance_variable(c"platform").unwrap();
        *ivar.load_ptr::<*mut c_void>(&*delegate) = Rc::as_ptr(&platform).cast_mut().cast();

        for visible in [Bool::NO, Bool::YES] {
            let result: Bool = msg_send![delegate,
                applicationShouldHandleReopen: std::ptr::null_mut::<AnyObject>(), hasVisibleWindows: visible];
            assert_eq!(
                result,
                Bool::YES,
                "AppKit handles reopen without a callback"
            );
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

        for (visible, expected_calls) in [(Bool::YES, 1), (Bool::NO, 11), (Bool::YES, 21)] {
            let result: Bool = msg_send![delegate,
                applicationShouldHandleReopen: std::ptr::null_mut::<AnyObject>(), hasVisibleWindows: visible];
            assert_eq!(result, Bool::NO, "the application callback owns reopen");
            assert_eq!(calls.get(), expected_calls);
        }

        *ivar.load_ptr::<*mut c_void>(&*delegate) = std::ptr::null_mut();
        let _: () = msg_send![delegate, release];
    }
    println!("reopen: native BOOL ABI, both visibility flags, callback replacement passed");
}
