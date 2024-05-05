/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGColorSpace.h`

use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::cf_string::CFStringRef;
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::core_graphics::CGFloat;
use crate::frameworks::foundation::ns_string;
use crate::frameworks::uikit::ui_color;
use crate::mem::{guest_size_of, MutPtr};
use crate::objc::{msg, objc_classes, ClassExports, HostObject};
use crate::Environment;


pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGColorSpace seems to be a CFType-based type, but in our implementation
// those are just Objective-C types, so we need a class for it, but its name is
// not visible anywhere.
@implementation _touchHLE_CGColorSpace: NSObject
@end

};

pub type CGColorSpaceModel = i32;
#[allow(dead_code)]
pub const kCGColorSpaceModelUnknown: CGColorSpaceModel = -1;
pub const kCGColorSpaceModelMonochrome: CGColorSpaceModel = 0;
pub const kCGColorSpaceModelRGB: CGColorSpaceModel = 1;
#[allow(dead_code)]
pub const kCGColorSpaceModelCMYK: CGColorSpaceModel = 2;
#[allow(dead_code)]
pub const kCGColorSpaceModelLab: CGColorSpaceModel = 3;
#[allow(dead_code)]
pub const kCGColorSpaceModelDeviceN: CGColorSpaceModel = 4;
#[allow(dead_code)]
pub const kCGColorSpaceModelIndexed: CGColorSpaceModel = 5;
#[allow(dead_code)]
pub const kCGColorSpaceModelPattern: CGColorSpaceModel = 6;

pub(super) struct CGColorSpaceHostObject {
    pub(super) name: &'static str,
}
impl HostObject for CGColorSpaceHostObject {}

pub type CGColorSpaceRef = CFTypeRef;
pub type CGColorRef = CFTypeRef;

pub fn CGColorSpaceCreateWithName(env: &mut Environment, name: CFStringRef) -> CGColorSpaceRef {
    let generic_rgb = ns_string::get_static_str(env, kCGColorSpaceGenericRGB);
    // TODO: support more color spaces
    assert!(msg![env; name isEqualToString:generic_rgb]);

    let isa = env
        .objc
        .get_known_class("_touchHLE_CGColorSpace", &mut env.mem);
    env.objc.alloc_object(
        isa,
        Box::new(CGColorSpaceHostObject {
            name: kCGColorSpaceGenericRGB,
        }),
        &mut env.mem,
    )
}

pub fn CGColorSpaceCreateDeviceRGB(env: &mut Environment) -> CGColorSpaceRef {
    // TODO: figure out what characteristics kCGColorSpaceDeviceRGB actually has
    //       on an iPhone
    let isa = env
        .objc
        .get_known_class("_touchHLE_CGColorSpace", &mut env.mem);
    env.objc.alloc_object(
        isa,
        Box::new(CGColorSpaceHostObject {
            name: kCGColorSpaceGenericRGB,
        }),
        &mut env.mem,
    )
}

fn CGColorSpaceCreateDeviceGray(env: &mut Environment) -> CGColorSpaceRef {
    let isa = env
        .objc
        .get_known_class("_touchHLE_CGColorSpace", &mut env.mem);
    env.objc.alloc_object(
        isa,
        Box::new(CGColorSpaceHostObject {
            name: kCGColorSpaceGenericGray,
        }),
        &mut env.mem,
    )
}

pub fn CGColorSpaceRelease(env: &mut Environment, cs: CGColorSpaceRef) {
    if !cs.is_null() {
        CFRelease(env, cs);
    }
}
pub fn CGColorSpaceRetain(env: &mut Environment, cs: CGColorSpaceRef) -> CGColorSpaceRef {
    if !cs.is_null() {
        CFRetain(env, cs)
    } else {
        cs
    }
}

pub fn CGColorSpaceGetModel(env: &mut Environment, cs: CGColorSpaceRef) -> CGColorSpaceModel {
    let host_object = env.objc.borrow::<CGColorSpaceHostObject>(cs);
    match host_object.name {
        kCGColorSpaceGenericGray => kCGColorSpaceModelMonochrome,
        kCGColorSpaceGenericRGB => kCGColorSpaceModelRGB,
        _ => unimplemented!(),
    }
}

pub type CGColorRef = CFTypeRef;

pub fn CGColorGetComponents(env: &mut Environment, color: CGColorRef) -> MutPtr<CGFloat> {
    let (r, g, b, a) = ui_color::get_rgba(&env.objc, color);
    let ptr: MutPtr<CGFloat> = env.mem.alloc(4 * guest_size_of::<CGFloat>()).cast();
    env.mem.write(ptr, r);
    env.mem.write(ptr + 1, g);
    env.mem.write(ptr + 2, b);
    env.mem.write(ptr + 3, a);
    ptr
}

pub fn CGColorGetColorSpace(env: &mut Environment, _color: CGColorRef) -> CGColorSpaceRef {
    // FIXME: what if a color is not sRGB?
    let srgb_name = ns_string::get_static_str(env, kCGColorSpaceGenericRGB);
    CGColorSpaceCreateWithName(env, srgb_name)
}

fn CGColorCreate(env: &mut Environment, cs: CGColorSpaceRef, components: MutPtr<CGFloat>) -> CGColorRef {
    let color_space = env.objc.borrow::<CGColorSpaceHostObject>(cs).name;
    assert_eq!(color_space, kCGColorSpaceGenericRGB);
    let r = env.mem.read(components);
    let g = env.mem.read(components + 1);
    let b = env.mem.read(components + 2);
    let a = env.mem.read(components + 3);
    msg_class![env; UIColor colorWithRed:r green:g blue:b alpha:a]
}

pub const kCGColorSpaceGenericRGB: &str = "kCGColorSpaceGenericRGB";
pub const kCGColorSpaceGenericGray: &str = "kCGColorSpaceGenericGray";

pub const CONSTANTS: ConstantExports = &[
    (
        "_kCGColorSpaceGenericRGB",
        HostConstant::NSString(kCGColorSpaceGenericRGB),
    ),
    (
        "_kCGColorSpaceGenericGray",
        HostConstant::NSString(kCGColorSpaceGenericGray),
    ),
];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGColorSpaceCreateWithName(_)),
    export_c_func!(CGColorSpaceCreateDeviceRGB()),
    export_c_func!(CGColorSpaceCreateDeviceGray()),
    export_c_func!(CGColorSpaceRetain(_)),
    export_c_func!(CGColorSpaceRelease(_)),
    export_c_func!(CGColorSpaceGetModel(_)),
    export_c_func!(CGColorGetComponents(_)),
    export_c_func!(CGColorGetColorSpace(_)),
    export_c_func!(CGColorGetComponents(_)),
    export_c_func!(CGColorGetColorSpace(_)),
    export_c_func!(CGColorCreate(_, _)),
];
