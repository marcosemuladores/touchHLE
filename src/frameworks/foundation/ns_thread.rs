/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSThread`.

use super::NSTimeInterval;
use crate::abi::GuestFunction;
use crate::dyld::HostFunction;
use crate::dyld::FunctionExports;
use crate::environment::{Environment, ThreadId};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::frameworks::foundation::{NSTimeInterval, NSUInteger};
use crate::libc::pthread::thread::{
    pthread_attr_init, pthread_attr_setdetachstate, pthread_attr_t, pthread_create, pthread_t,
    PTHREAD_CREATE_DETACHED,
};
use crate::mem::{guest_size_of, ConstPtr, Mem, MutPtr};
use crate::msg;
use crate::objc::{
    id, msg_send, nil, objc_classes, release, retain, Class, ClassExports, HostObject, NSZonePtr,
    SEL,
};
use crate::{export_c_func};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Default)]
pub struct State {
    thread_start_fn: Option<GuestFunction>,
    thread_map: HashMap<ThreadId, id>,
}

/// Belongs to NSThread
struct ThreadHostObject {
    target: id,
    selector: SEL,
    argument: id,
    stack_size: NSUInteger,
    run_loop: id,
    is_cancelled: bool,
}
impl HostObject for ThreadHostObject {}
struct NSThreadHostObject {
    target: id,
    selector: Option<SEL>,
    object: id,
}
impl HostObject for NSThreadHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSThread: NSObject

+ (f64)threadPriority {
    log!("TODO: [NSThread threadPriority] (not implemented yet)");
    1.0
}

+ (bool)setThreadPriority:(f64)priority {
    log!("TODO: [NSThread setThreadPriority:{:?}] (ignored)", priority);
    true
}


+ (id)mainThread {
    #[allow(clippy::map_entry)]
    if !env.framework_state.foundation.ns_thread.thread_map.contains_key(&0) {
        let thread = msg![env; this alloc];
        let r_loop = env.objc.borrow_mut::<ThreadHostObject>(thread).run_loop;
        () = msg![env; r_loop _setMainThread];
        env.framework_state.foundation.ns_thread.thread_map.insert(0, thread);
    }
    *env.framework_state.foundation.ns_thread.thread_map.get(&0).unwrap()
}
    
+ (id)currentThread {
    if env.current_thread == 0 {
        msg![env; this mainThread]
    } else {
        *env.framework_state.foundation.ns_thread.thread_map.get(&env.current_thread).unwrap()
    }
}

+ (id)allocWithZone:(NSZonePtr)_zone {
    let r_loop = msg_class![env; NSRunLoop alloc];
    let host_object = Box::new(ThreadHostObject {
        target: nil,
        argument: nil,
        selector: SEL::null(),
        stack_size: Mem::SECONDARY_THREAD_STACK_SIZE,
        run_loop: r_loop,
        is_cancelled: false
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (())detachNewThreadSelector:(SEL)selector
                       toTarget:(id)target
                     withObject:(id)object {
    let host_object = Box::new(NSThreadHostObject {
        target,
        selector: Some(selector),
        object,
    });
    let this = env.objc.alloc_object(this, host_object, &mut env.mem);
    retain(env, this);

    retain(env, target);
    retain(env, object);

    let symb = "__ns_thread_invocation";
    let gf = env
        .dyld
        .create_private_proc_address(&mut env.mem, &mut env.cpu, symb)
        .unwrap_or_else(|_| panic!("create_private_proc_address failed {}", symb));

    let attr: MutPtr<pthread_attr_t> = env.mem.alloc(guest_size_of::<pthread_attr_t>()).cast();
    pthread_attr_init(env, attr);

    pthread_attr_setdetachstate(env, attr, PTHREAD_CREATE_DETACHED);
    let thread_ptr: MutPtr<pthread_t> = env.mem.alloc(guest_size_of::<pthread_t>()).cast();

    pthread_create(env, thread_ptr, attr.cast_const(), gf, this.cast());

    // TODO: post NSWillBecomeMultiThreadedNotification
}
    
+ (())sleepForTimeInterval:(NSTimeInterval)ti {
    log_dbg!("[NSThread sleepForTimeInterval:{:?}]", ti);
    env.sleep(Duration::from_secs_f64(ti), /* tail_call: */ true);
}

+ (bool)isCancelled {
    false
}

+ (())detachNewThreadSelector:(SEL)selector
                       toTarget:(id)target
                     withObject:(id)object {
    let host_object = Box::new(NSThreadHostObject {
        target,
        selector: Some(selector),
        object,
    });
    let this = env.objc.alloc_object(this, host_object, &mut env.mem);
    retain(env, this);

    retain(env, target);
    retain(env, object);

    let symb = "__ns_thread_invocation";
    let hf: HostFunction = &(_ns_thread_invocation as fn(&mut Environment, _) -> _);
    let gf = env
        .dyld
        .create_guest_function(&mut env.mem, symb, hf);

    let attr: MutPtr<pthread_attr_t> = env.mem.alloc(guest_size_of::<pthread_attr_t>()).cast();
    pthread_attr_init(env, attr);

    pthread_attr_setdetachstate(env, attr, PTHREAD_CREATE_DETACHED);
    let thread_ptr: MutPtr<pthread_t> = env.mem.alloc(guest_size_of::<pthread_t>()).cast();

    pthread_create(env, thread_ptr, attr.cast_const(), gf, this.cast());

    // TODO: post NSWillBecomeMultiThreadedNotification
}

// TODO: construction etc

+ (())exit {

}

- (id)initWithTarget:(id)target selector:(SEL)selector object:(id)object {
    let host_object: &mut NSThreadHostObject = env.objc.borrow_mut(this);
    host_object.target = target;
    host_object.selector = Some(selector);
    host_object.object = object;
    this
}

- (())start {
    let symb = "__ns_thread_invocation";
    let gf = env
        .dyld
        .create_private_proc_address(&mut env.mem, &mut env.cpu, symb)
        .unwrap_or_else(|_| panic!("create_private_proc_address failed {}", symb));

    let thread_ptr: MutPtr<pthread_t> = env.mem.alloc(guest_size_of::<pthread_t>()).cast();
    pthread_create(env, thread_ptr, ConstPtr::null(), gf, this.cast());
}
    
@end

};

type NSThreadRef = CFTypeRef;

pub fn get_run_loop(env: &mut Environment, thread: id) -> id {
    env.objc.borrow::<ThreadHostObject>(thread).run_loop
}

fn _NSThreadStart(env: &mut Environment, thread: id) {
    () = msg![env; thread main];
    release(env, thread);
}

pub fn _ns_thread_invocation(env: &mut Environment, ns_thread_obj: NSThreadRef) {
    let class: Class = msg![env; ns_thread_obj class];
    log_dbg!(
        "_ns_thread_invocation on object of class: {}",
        env.objc.get_class_name(class)
    );
    assert_eq!(class, env.objc.get_known_class("NSThread", &mut env.mem));

    let &NSThreadHostObject {
        target,
        selector,
        object,
    } = env.objc.borrow(ns_thread_obj);
    () = msg_send(env, (target, selector.unwrap(), object));

    release(env, object);
    release(env, target);

    release(env, ns_thread_obj);

    // TODO: NSThread exit
}

pub const PRIVATE_FUNCTIONS: FunctionExports = &[
    export_c_func!(_ns_thread_invocation(_))];
    export_c_func!(_NSThreadStart(_))];
