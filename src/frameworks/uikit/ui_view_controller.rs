/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIViewController`.

use crate::frameworks::foundation::ns_string::{get_static_str, to_rust_string};
use crate::objc::{
    id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};

#[derive(Default)]
struct UIViewControllerHostObject {
    view: id,
}
impl HostObject for UIViewControllerHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIViewController: UIResponder

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UIViewControllerHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: this should be the designated initializer
- (id)initWithNibName:(id)nibNameOrNil // NSString *
               bundle:(id)nibBundleOrNil { // NSBundle *
    assert!(nibBundleOrNil == nil);
    if (nibNameOrNil == nil) {
        return this;
    }
    let x = to_rust_string(env, nibNameOrNil);
    log!("initWithNibName: {}", x);
    let bundle: id = msg_class![env; NSBundle mainBundle];
    let type_: id = get_static_str(env, "nib");
    let path: id = msg![env; bundle pathForResource:nibNameOrNil ofType:type_];

    assert!(msg![env; path isAbsolutePath]);
    let ns_data: id = msg_class![env; NSData dataWithContentsOfFile:path];
    assert!(ns_data != nil);

    let unarchiver = msg_class![env; NSKeyedUnarchiver alloc];
    let unarchiver = msg![env; unarchiver initForReadingWithData:ns_data];

    // We don't need to do anything with the list of objects, but deserializing
    // it ensures everything else is deserialized.
    let objects_key = get_static_str(env, "UINibObjectsKey");
    let _objects: id = msg![env; unarchiver decodeObjectForKey:objects_key];

    msg![env; this initWithCoder:unarchiver]
}

- (id)initWithCoder:(id)coder {
    let key_ns_string = get_static_str(env, "UIView");
    let view: id = msg![env; coder decodeObjectForKey:key_ns_string];

    () = msg![env; this setView:view];

    this
}

- (())dealloc {
    let &UIViewControllerHostObject { view } = env.objc.borrow(this);

    release(env, view);

    env.objc.dealloc_object(this, &mut env.mem);
}

- (())loadView {
    // TODO: Check if the UIViewController has an associated nib file and load
    // the view from there instead if it does
    let view: id = msg_class![env; UIView alloc];
    let view: id = msg![env; view init];
    () = msg![env; this setView: view];
}
- (())setView:(id)new_view { // UIView*
    let host_obj = env.objc.borrow_mut::<UIViewControllerHostObject>(this);
    let old_view = std::mem::replace(&mut host_obj.view, new_view);
    retain(env, new_view);
    release(env, old_view);
}
- (id)view {
    let view = env.objc.borrow_mut::<UIViewControllerHostObject>(this).view;
    if view == nil {
        () = msg![env; this loadView];
        let view = env.objc.borrow_mut::<UIViewControllerHostObject>(this).view;
        view
    } else {
        view
    }
}

- (())setEditing:(bool)editing {
    log!("TODO: [(UIViewController*){:?} setEditing:{}]", this, editing); // TODO
}

- (())dismissModalViewControllerAnimated:(bool)animated {
    log!("TODO: [(UIViewController*){:?} dismissModalViewControllerAnimated:{}]", this, animated); // TODO
}
    
@end

};
