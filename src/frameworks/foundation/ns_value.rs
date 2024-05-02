/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSValue` class cluster, including `NSNumber`.

use super::{
    NSComparisonResult, NSInteger, NSOrderedAscending, NSOrderedDescending, NSOrderedSame, NSUInteger,
};
use crate::frameworks::foundation::ns_string::from_rust_string;
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, retain, Class, ClassExports, HostObject,
    NSZonePtr,
};
use std::cmp::Ordering;

#[derive(Debug)]
pub enum NSNumberHostObject {
    Bool(bool),
    Int(i32),
    UnsignedLongLong(u64),
    LongLong(i64),
    Float(f32),
    Double(f64),
}
impl HostObject for NSNumberHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSValue is an abstract class. None of the things it should provide are
// implemented here yet (TODO).
@implementation NSValue: NSObject

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

@end

// NSNumber is not an abstract class.
@implementation NSNumber: NSValue

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberHostObject::Bool(false));
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)numberWithBool:(bool)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBool:value];
    autorelease(env, new)
}

+ (id)numberWithInteger:(NSInteger)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInteger:value];
    autorelease(env, new)
}

+ (id)numberWithInt:(i32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInt:value];
    autorelease(env, new)
}

+ (id)numberWithFloat:(f32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithFloat:value];
    autorelease(env, new)
}

+ (id)numberWithDouble:(f64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithDouble:value];
    autorelease(env, new)
}

+ (id)numberWithLongLong:(i64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithLongLong:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedLongLong:(u64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedLongLong:value];
    autorelease(env, new)
}

// TODO: types other than booleans and long longs

- (id)initWithBool:(bool)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Bool(value);
    this
}

- (id)initWithInteger:(NSInteger)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithInt:(i32)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithFloat:(f32)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Float(value);
    this
}

- (id)initWithDouble:(f64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Double(value);
    this
}

- (id)initWithLongLong:(i64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::LongLong(value);
    this
}

- (id)initWithUnsignedLongLong:(u64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedLongLong(value);
    this
}

- (id)description {
    match env.objc.borrow(this) {
        NSNumberHostObject::Bool(value) => from_rust_string(env, (*value as i32).to_string()),
        NSNumberHostObject::Int(value) => from_rust_string(env, value.to_string()),
        NSNumberHostObject::UnsignedLongLong(value) => from_rust_string(env, value.to_string()),
        NSNumberHostObject::LongLong(value) => from_rust_string(env, value.to_string()),
        NSNumberHostObject::Float(value) => from_rust_string(env, value.to_string()),
        NSNumberHostObject::Double(value) => from_rust_string(env, value.to_string())
    }
}
- (NSUInteger)hash {
    let &NSNumberHostObject::Bool(value) = env.objc.borrow(this) else {
        todo!();
    };
    super::hash_helper(&value)
}
- (bool)isEqualTo:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSNumber class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
    let &NSNumberHostObject::Bool(a) = env.objc.borrow(this) else {
        todo!();
    };
    let &NSNumberHostObject::Bool(b) = env.objc.borrow(other) else {
        todo!();
    };
    a == b
}

- (f64)doubleValue {
    let value = if let &NSNumberHostObject::Float(value) = env.objc.borrow(this) { value } else { todo!() };
    value.try_into().unwrap()
}
    
// TODO: accessors etc

- (NSInteger)integerValue {
    let value = if let &NSNumberHostObject::Int(value) = env.objc.borrow(this) { value } else { todo!() };
    value
}
- (i32)intValue {
    msg![env; this integerValue]
}
- (f32)floatValue {
    let value = if let &NSNumberHostObject::Float(value) = env.objc.borrow(this) { value } else { todo!() };
    value
}
- (bool)boolValue {
    let value = if let &NSNumberHostObject::Bool(value) = env.objc.borrow(this) { value } else { todo!() };
    value
}
-(ConstPtr<u8>)objCType {
    let ty = match env.objc.borrow::<NSNumberHostObject>(this) {
        NSNumberHostObject::Bool(_) => "B",
        NSNumberHostObject::UnsignedLongLong(_) => "Q",
        NSNumberHostObject::LongLong(_) => "q",
        NSNumberHostObject::Double(_) => "d",
    };
    let c_string = env.mem.alloc_and_write_cstr(ty.as_bytes());
    let length: NSUInteger = (ty.len() + 1).try_into().unwrap();
    // NSData will handle releasing the string (it is autoreleased)
    let _: id = msg_class![env; NSData dataWithBytesNoCopy:(c_string.cast_void())
                                                    length:length];
    c_string.cast_const()
}

- (NSComparisonResult)compare:(id)other {
    match *env.objc.borrow::<NSNumberHostObject>(this) {
        NSNumberHostObject::Bool(v) => {
            let other_v: bool = msg![env; other boolValue];
            if !v && other_v {
                NSOrderedAscending
            } else if v == other_v {
                NSOrderedSame
            } else {
                NSOrderedDescending
            }
        }
        NSNumberHostObject::UnsignedLongLong(v) => {
            let other_v: u64 = msg![env; other unsignedLongLongValue];
            match v.cmp(&other_v) {
                Ordering::Less => NSOrderedAscending,
                Ordering::Equal => NSOrderedSame,
                Ordering::Greater => NSOrderedDescending
            }
        },
        NSNumberHostObject::LongLong(v) => {
            let other_v: i64 = msg![env; other longLongValue];
            match v.cmp(&other_v) {
                Ordering::Less => NSOrderedAscending,
                Ordering::Equal => NSOrderedSame,
                Ordering::Greater => NSOrderedDescending
            }
        },
        NSNumberHostObject::Double(v) => {
            let other_v: f64 = msg![env; other doubleValue];
            if v < other_v {
                NSOrderedAscending
            } else if v == other_v {
                NSOrderedSame
            } else {
                NSOrderedDescending
            }
        },
    }
}

@end

};
