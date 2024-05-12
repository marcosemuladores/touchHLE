/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSDate`.

use std::time;
use super::NSTimeInterval;
use crate::frameworks::core_foundation::time::apple_epoch;
use crate::frameworks::foundation::{ns_string, NSInteger};
use crate::objc::{autorelease, id, msg, msg_class, objc_classes, ClassExports, HostObject};

use std::time::SystemTime;

struct NSTimeZoneHostObject {
    _time_zone: String,
}
impl HostObject for NSTimeZoneHostObject {}

struct NSDateHostObject {
    instant: SystemTime,
}
impl HostObject for NSDateHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSDate: NSObject

+ (id)date {
    // "Date objects are immutable, representing an invariant time interval
    // relative to an absolute reference date (00:00:00 UTC on 1 January 2001)."
    let time_interval = SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64();
    let host_object = Box::new(NSDateHostObject {
        instant: SystemTime::now()
    });
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    log_dbg!("[NSDate date] => {:?} ({:?}s)", new, time_interval);
    autorelease(env, new)
}

+ (id)distantFuture {
    let time_interval = SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64() * 2.0;
    let host_object = Box::new(NSDateHostObject {
        time_interval
    });
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);

    log_dbg!("[(NSDate*){:?} distantFuture]: date {:?}", this, new);

    autorelease(env, new)
}
    
+ (NSTimeInterval)timeIntervalSinceReferenceDate {
    let now: id = msg_class![env; NSDate date];
    msg![env; now timeIntervalSinceReferenceDate]
}

- (NSTimeInterval)timeIntervalSinceDate:(id)anotherDate {
    assert!(!anotherDate.is_null());
    let host_object = env.objc.borrow::<NSDateHostObject>(this);
    let another_date_host_object = env.objc.borrow::<NSDateHostObject>(anotherDate);
    let result =  host_object.time_interval-another_date_host_object.time_interval;
    log_dbg!("[(NSDate*){:?} ({:?}s) timeIntervalSinceDate:{:?} ({:?}s)] => {}", this, host_object.time_interval, anotherDate, another_date_host_object.time_interval, result);
    result
}

- (NSTimeInterval)timeIntervalSinceReferenceDate {
    env.objc.borrow::<NSDateHostObject>(this).time_interval
}

- (NSTimeInterval)timeIntervalSince1970 { 
    let host_object = env.objc.borrow::<NSDateHostObject>(this); 
    let result = host_object.instant + 978_307_200.0; 
    result
}    

- (NSTimeInterval)timeIntervalSinceNow {

    let host_object = env.objc.borrow::<NSDateHostObject>(this);

    let time_interval = SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64();

    time_interval - host_object.time_interval

}

- (id)addTimeInterval:(NSTimeInterval)seconds {
    let interval = env.objc.borrow::<NSDateHostObject>(this).time_interval + seconds;
    let date = msg_class![env; NSDate date];
    env.objc.borrow_mut::<NSDateHostObject>(date).time_interval = interval;
    date
}

@end

@implementation NSTimeZone: NSObject

+ (id)timeZoneWithName:(id)tzName {
    let time_zone = ns_string::to_rust_string(env, tzName);
    let host_object = NSTimeZoneHostObject {
        _time_zone: time_zone.to_string(),
    };
    env.objc.alloc_object(this, Box::new(host_object), &mut env.mem)
}

+ (id)localTimeZone {
    let host_object = NSTimeZoneHostObject {
        _time_zone: "UTC".to_string(),
    };
    env.objc.alloc_object(this, Box::new(host_object), &mut env.mem)
}

- (NSInteger)secondsFromGMT {
    0
}
    
@end

};
