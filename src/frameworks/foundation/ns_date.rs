/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSDate`.

use super::NSTimeInterval;
use crate::frameworks::core_foundation::time::apple_epoch;
use crate::objc::{autorelease, id, msg, msg_class, objc_classes, ClassExports, HostObject, NSZonePtr};

use std::ops::Add;
use std::time::{Duration, Instant, SystemTime};

struct NSDateHostObject {
    time_interval: NSTimeInterval,
}
impl HostObject for NSDateHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSDate: NSObject

+ (NSTimeInterval)timeIntervalSinceReferenceDate {
    SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64()
}

+ (id)allocWithZone:(NSZonePtr)_zone {
    // "Date objects are immutable, representing an invariant time interval
    // relative to an absolute reference date (00:00:00 UTC on 1 January 2001)."
    let time_interval = SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64();
    let host_object = Box::new(NSDateHostObject {
        time_interval
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)date {
    let new = msg![env; this alloc];
    log_dbg!("[NSDate date] => {:?} ({:?}s)", new, time_interval);
    autorelease(env, new)
}
                          
+ (NSTimeInterval)timeIntervalSinceReferenceDate {
    let now: id = msg_class![env; NSDate date];
    msg![env; now timeIntervalSinceReferenceDate]
}

- (id)init {
    this
}

- (id)initWithTimeIntervalSinceNow:(NSTimeInterval)secs {
    let host_object = env.objc.borrow_mut::<NSDateHostObject>(this);
    host_object.instant = Instant::now() + Duration::from_secs_f64(secs);
    this
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

- (NSTimeInterval)timeIntervalSinceNow {
    let host_object = env.objc.borrow::<NSDateHostObject>(this);
    let time_interval = SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64();
    time_interval - host_object.time_interval
}

- (NSTimeInterval)timeIntervalSince1970 {
    let time_interval = env.objc.borrow::<NSDateHostObject>(this).time_interval;
    apple_epoch()
        .add(Duration::from_secs_f64(time_interval))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

- (id)addTimeInterval:(NSTimeInterval)seconds {
    let interval = env.objc.borrow::<NSDateHostObject>(this).time_interval + seconds;
    let date = msg_class![env; NSDate date];
    env.objc.borrow_mut::<NSDateHostObject>(date).time_interval = interval;
    date
}

@end

};
