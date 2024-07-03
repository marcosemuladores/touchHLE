/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSThread`.

use super::NSTimeInterval
use crate::abi::GuestFunction;
use crate::dyld::{FunctionExports, HostFunction};
use crate::environment::{Environment, ThreadId};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::libc::pthread::thread::{
    _get_thread_by_id, _get_thread_id, pthread_attr_init, pthread_attr_setdetachstate,
    pthread_attr_t, pthread_create, pthread_t, PTHREAD_CREATE_DETACHED,
};
use crate::mem::{guest_size_of, ConstPtr, Men, MutPtr};
use crate::objc::{
    id, msg_send, nil, objc_classes, release, retain, Class, ClassExports, HostObject, NSZonePtr,
    SEL,
};
use crate::{export_c_func, msg, msg_class};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Default)]
pub struct State {
    /// `NSThread*`
    ns_threads: HashSet<id>ThreadId, id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut State {
        &mut env.framework_state.foundation.ns_threads
    }
}

struct NSThreadHostObject {
    thread: Option<pthread_t>,
    target: id,
    selector: Option<SEL>,
    argument: id,
    stack_size: NSUInteger,
    run_loop: id,
    is_cancelled: bool,
    object: id,
    /// `NSMutableDictionary*`
    thread_dictionary: id,
}
impl HostObject for NSThreadHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSThread: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    log_dbg!("[NSThread allocWithZone:{:?}]", zone);
    let host_object = NSThreadHostObject { thread: None, target: nil, selector: None, object: nil, thread_dictionary: nil };
    let guest_object = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
    State::get(env).ns_threads.insert(guest_object);
    guest_object
}

+ (f64)threadPriority {
    let current_thread = msg_class![env; NSThread currentThread];
    msg![env; current_thread threadPriority]
}

+ (bool)setThreadPriority:(f64)priority {
    let current_thread = msg_class![env; NSThread currentThread];
    msg![env; current_thread setThreadPriority:priority]
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

+ (())sleepForTimeInterval:(NSTimeInterval)ti {
    log_dbg!("[NSThread sleepForTimeInterval:{:?}]", ti);
    env.sleep(Duration::from_secs_f64(ti), /* tail_call: */ true);
}

+ (())detachNewThreadSelector:(SEL)selector
                       toTarget:(id)target
                     withObject:(id)object {
    let host_object = Box::new(NSThreadHostObject {
        thread: None,
        target,
        selector: Some(selector),
        object,
        thread_dictionary: nil,
    });
    let this = env.objc.alloc_object(this, host_object, &mut env.mem);
    retain(env, this);

    retain(env, target);
    retain(env, object);

    let symb = "__touchHLE_NSThreadInvocationHelper";
    let hf: HostFunction = &(_touchHLE_NSThreadInvocationHelper as fn(&mut Environment, _) -> _);
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

- (id)initWithTarget:(id)target
selector:(SEL)selector
object:(id)object {
    let host_object: &mut NSThreadHostObject = env.objc.borrow_mut(this);
    host_object.target = target;
    host_object.selector = Some(selector);
    host_object.object = object;
    this
}

- (())start {
    let symb = "__touchHLE_NSThreadInvocationHelper";
    let hf: HostFunction = &(_touchHLE_NSThreadInvocationHelper as fn(&mut Environment, _) -> _);
    let gf = env
        .dyld
        .create_guest_function(&mut env.mem, symb, hf);

    let thread_ptr: MutPtr<pthread_t> = env.mem.alloc(guest_size_of::<pthread_t>()).cast();
    pthread_create(env, thread_ptr, ConstPtr::null(), gf, this.cast());
    let thread = env.mem.read(thread_ptr);
    // TODO: Store the thread's default NSConnection
    // and NSAssertionHandler instances
    // https://developer.apple.com/documentation/foundation/nsthread/1411433-threaddictionary

    let host_object = env.objc.borrow_mut::<NSThreadHostObject>(this);
    host_object.thread = Some(thread);
    host_object.thread_dictionary = nil;

    log_dbg!("[(NSThread*){:?} start] Started new thread with pthread {:?} and ThreadId {:?}", this, thread, _get_thread_id(env, thread));
}

- (())main {
    let host = env.objc.borrow::<ThreadHostObject>(this);
    () = msg_send(env, (host.target, host.selector, host.argument));
}

- (id)threadDictionary {
    // Initialize lazily in case the thread is started with pthread_create
    let thread_dictionary = env.objc.borrow::<NSThreadHostObject>(this).thread_dictionary;
    if thread_dictionary == nil {
        let thread_dictionary = msg_class![env; NSMutableDictionary new];
        // TODO: Store the thread's default NSConnection
        // and NSAssertionHandler instances
        // https://developer.apple.com/documentation/foundation/nsthread/1411433-threaddictionary
        env.objc.borrow_mut::<NSThreadHostObject>(this).thread_dictionary = thread_dictionary;
        thread_dictionary
    } else {
        thread_dictionary
    }
}

- (f64)threadPriority {
    log!("TODO: [(NSThread*){:?} threadPriority] (not implemented yet)", this);
    1.0
}

- (bool)setThreadPriority:(f64)priority {
    log!("TODO: [(NSThread*){:?} setThreadPriority:{:?}] (ignored)", this, priority);
    true
}

- (NSUInteger)stackSize {
    env.objc.borrow::<ThreadHostObject>(this).stack_size
}

-(())setStackSize:(NSUInteger)size {
    env.objc.borrow_mut::<ThreadHostObject>(this).stack_size = size;
}

- (bool) isCancelled {
    env.objc.borrow::<ThreadHostObject>(this).is_cancelled
}

- (())dealloc {
    log_dbg!("[(NSThread*){:?} dealloc]", this);
    State::get(env).ns_threads.remove(&this);
    let _host_object = env.objc.borrow::<NSThreadHostObject>(this);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};


pub fn get_run_loop(env: &mut Environment, thread: id) -> id {
    env.objc.borrow::<ThreadHostObject>(thread).run_loop
}

fn _NSThreadStart(env: &mut Environment, thread: id) {
    () = msg![env; thread main];
    release(env, thread);
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(_NSThreadStart(_))];
    
type NSThreadRef = CFTypeRef;

pub fn _touchHLE_NSThreadInvocationHelper(env: &mut Environment, ns_thread_obj: NSThreadRef) {
    let class: Class = msg![env; ns_thread_obj class];
    log_dbg!(
        "_touchHLE_NSThreadInvocationHelper on object of class: {}",
        env.objc.get_class_name(class)
    );
    assert_eq!(class, env.objc.get_known_class("NSThread", &mut env.mem));

    let &NSThreadHostObject {
        thread: _,
        target,
        selector,
        object,
        thread_dictionary: _,
    } = env.objc.borrow(ns_thread_obj);
    () = msg_send(env, (target, selector.unwrap(), object));

    release(env, object);
    release(env, target);

    release(env, ns_thread_obj);

    // TODO: NSThread exit
}
