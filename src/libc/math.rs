/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `math.h`

use std::num::FpCategory;
use crate::dyld::{export_c_func, FunctionExports};
use crate::Environment;
use crate::mem::{MutPtr, MutVoidPtr};

// The sections in this file are organized to match the C standard.

// FIXME: Many functions in this file should theoretically set errno or affect
//        the floating-point environment. We're hoping apps won't rely on that.

// Trigonometric functions

// TODO: These should also have `long double` variants, which can probably just
// alias the `double` ones.

fn sin(_env: &mut Environment, arg: f64) -> f64 {
    arg.sin()
}
fn sinf(_env: &mut Environment, arg: f32) -> f32 {
    arg.sin()
}
fn cos(_env: &mut Environment, arg: f64) -> f64 {
    arg.cos()
}
fn cosf(_env: &mut Environment, arg: f32) -> f32 {
    arg.cos()
}
fn tan(_env: &mut Environment, arg: f64) -> f64 {
    arg.tan()
}
fn tanf(_env: &mut Environment, arg: f32) -> f32 {
    arg.tan()
}

fn asin(_env: &mut Environment, arg: f64) -> f64 {
    arg.asin()
}
fn asinf(_env: &mut Environment, arg: f32) -> f32 {
    arg.asin()
}
fn acos(_env: &mut Environment, arg: f64) -> f64 {
    arg.acos()
}
fn acosf(_env: &mut Environment, arg: f32) -> f32 {
    arg.acos()
}
fn atan(_env: &mut Environment, arg: f64) -> f64 {
    arg.atan()
}
fn atanf(_env: &mut Environment, arg: f32) -> f32 {
    arg.atan()
}

fn atan2f(_env: &mut Environment, arg1: f32, arg2: f32) -> f32 {
    arg1.atan2(arg2)
}
fn atan2(_env: &mut Environment, arg1: f64, arg2: f64) -> f64 {
    arg1.atan2(arg2)
}

// Hyperbolic functions

fn sinh(_env: &mut Environment, arg: f64) -> f64 {
    arg.sinh()
}
fn sinhf(_env: &mut Environment, arg: f32) -> f32 {
    arg.sinh()
}
fn cosh(_env: &mut Environment, arg: f64) -> f64 {
    arg.cosh()
}
fn coshf(_env: &mut Environment, arg: f32) -> f32 {
    arg.cosh()
}
fn tanh(_env: &mut Environment, arg: f64) -> f64 {
    arg.tanh()
}
fn tanhf(_env: &mut Environment, arg: f32) -> f32 {
    arg.tanh()
}

fn asinh(_env: &mut Environment, arg: f64) -> f64 {
    arg.asinh()
}
fn asinhf(_env: &mut Environment, arg: f32) -> f32 {
    arg.asinh()
}
fn acosh(_env: &mut Environment, arg: f64) -> f64 {
    arg.acosh()
}
fn acoshf(_env: &mut Environment, arg: f32) -> f32 {
    arg.acosh()
}
fn atanh(_env: &mut Environment, arg: f64) -> f64 {
    arg.atanh()
}
fn atanhf(_env: &mut Environment, arg: f32) -> f32 {
    arg.atanh()
}

// Exponential and logarithmic functions
// TODO: implement the rest
fn log(_env: &mut Environment, arg: f64) -> f64 {
    arg.ln()
}
fn logf(_env: &mut Environment, arg: f32) -> f32 {
    arg.ln()
}
fn log1p(_env: &mut Environment, arg: f64) -> f64 {
    arg.ln_1p()
}
fn log1pf(_env: &mut Environment, arg: f32) -> f32 {
    arg.ln_1p()
}
fn log2(_env: &mut Environment, arg: f64) -> f64 {
    arg.log2()
}
fn log2f(_env: &mut Environment, arg: f32) -> f32 {
    arg.log2()
}
fn log10(_env: &mut Environment, arg: f64) -> f64 {
    arg.log10()
}
fn log10f(_env: &mut Environment, arg: f32) -> f32 {
    arg.log10()
}
fn exp(_env: &mut Environment, arg: f64) -> f64 {
    arg.exp()
}
fn expf(_env: &mut Environment, arg: f32) -> f32 {
    arg.exp()
}
fn expm1(_env: &mut Environment, arg: f64) -> f64 {
    arg.exp_m1()
}
fn expm1f(_env: &mut Environment, arg: f32) -> f32 {
    arg.exp_m1()
}
fn exp2(_env: &mut Environment, arg: f64) -> f64 {
    arg.exp2()
}
fn exp2f(_env: &mut Environment, arg: f32) -> f32 {
    arg.exp2()
}

// Power functions
// TODO: implement the rest
fn pow(_env: &mut Environment, arg1: f64, arg2: f64) -> f64 {
    arg1.powf(arg2)
}
fn powf(_env: &mut Environment, arg1: f32, arg2: f32) -> f32 {
    arg1.powf(arg2)
}
fn sqrt(_env: &mut Environment, arg: f64) -> f64 {
    arg.sqrt()
}
fn sqrtf(_env: &mut Environment, arg: f32) -> f32 {
    arg.sqrt()
}

// Nearest integer functions
// TODO: implement the rest
fn ceil(_env: &mut Environment, arg: f64) -> f64 {
    arg.ceil()
}
fn ceilf(_env: &mut Environment, arg: f32) -> f32 {
    arg.ceil()
}
fn floor(_env: &mut Environment, arg: f64) -> f64 {
    arg.floor()
}
fn floorf(_env: &mut Environment, arg: f32) -> f32 {
    arg.floor()
}
fn round(_env: &mut Environment, arg: f64) -> f64 {
    arg.round()
}
fn roundf(_env: &mut Environment, arg: f32) -> f32 {
    arg.round()
}
fn trunc(_env: &mut Environment, arg: f64) -> f64 {
    arg.trunc()
}
fn truncf(_env: &mut Environment, arg: f32) -> f32 {
    arg.trunc()
}
fn modff(env: &mut Environment, val: f32, iptr: MutPtr<f32>) -> f32 {
    let ivalue = truncf(env, val);
    env.mem.write(iptr, ivalue);
    val - ivalue
}

// Remainder functions
// TODO: implement the rest
fn fmod(_env: &mut Environment, arg1: f64, arg2: f64) -> f64 {
    arg1 % arg2
}
fn fmodf(_env: &mut Environment, arg1: f32, arg2: f32) -> f32 {
    arg1 % arg2
}

// Maximum, minimum and positive difference functions
// TODO: implement fdim
fn fmax(_env: &mut Environment, arg1: f64, arg2: f64) -> f64 {
    arg1.max(arg2)
}
fn fmaxf(_env: &mut Environment, arg1: f32, arg2: f32) -> f32 {
    arg1.max(arg2)
}
fn fmin(_env: &mut Environment, arg1: f64, arg2: f64) -> f64 {
    arg1.min(arg2)
}
fn fminf(_env: &mut Environment, arg1: f32, arg2: f32) -> f32 {
    arg1.min(arg2)
}

fn lrintf(_env: &mut Environment, arg1: f32) -> i32 {
    arg1.max(i32::MIN as f32).min(i32::MAX as f32).round() as i32
}

type GuestFPCategory = i32;
const FP_NAN: GuestFPCategory = 1;
const FP_INFINITE: GuestFPCategory = 1;
const FP_ZERO: GuestFPCategory = 3;
const FP_NORMAL: GuestFPCategory = 4;
const FP_SUBNORMAL: GuestFPCategory = 4;

fn __fpclassifyf(_env: &mut Environment, arg: f32) -> GuestFPCategory {
    match arg.classify() {
        FpCategory::Nan => FP_NAN,
        FpCategory::Infinite => FP_INFINITE,
        FpCategory::Zero => FP_ZERO,
        FpCategory::Normal => FP_NORMAL,
        FpCategory::Subnormal => FP_SUBNORMAL
    }
}

// int32_t
//      OSAtomicAdd32Barrier(int32_t theAmount, volatile int32_t *theValue)
fn OSAtomicAdd32Barrier(
    env: &mut Environment, the_amount: i32, the_value: MutPtr<i32>
) -> i32 {
    let curr = env.mem.read(the_value);
    let new = curr + the_amount;
    env.mem.write(the_value, new);
    new
}

fn OSAtomicCompareAndSwap32Barrier(
    env: &mut Environment, old_value: i32, new_value: i32, the_value: MutPtr<i32>
) -> bool {
    if old_value == env.mem.read(the_value) {
        env.mem.write(the_value, new_value);
        true
    } else {
        false
    }
}

// bool
//      OSAtomicCompareAndSwapPtr(void* oldValue, void* newValue, void* volatile *theValue);
fn OSAtomicCompareAndSwapPtr(
    env: &mut Environment, old_value: MutVoidPtr, new_value: MutVoidPtr, the_value: MutPtr<MutVoidPtr>
) -> bool {
    if old_value == env.mem.read(the_value) {
        env.mem.write(the_value, new_value);
        true
    } else {
        false
    }
}

// int32_t	OSAtomicAdd32( int32_t __theAmount, volatile int32_t *__theValue );
fn OSAtomicAdd32(env: &mut Environment, amount: i32, value_ptr: MutPtr<i32>) -> i32 {
    let value = env.mem.read(value_ptr);
    let new_value = value + amount;
    env.mem.write(value_ptr, new_value);
    new_value
}

type OSSpinLock = i32;

// void    OSSpinLockLock( volatile OSSpinLock *__lock );
fn OSSpinLockLock(env: &mut Environment, lock: MutPtr<OSSpinLock>) {
    let curr = env.mem.read(lock);
    assert_eq!(curr, 0);
    env.mem.write(lock, 1);
}

fn OSSpinLockUnlock(env: &mut Environment, lock: MutPtr<OSSpinLock>) {
    let curr = env.mem.read(lock);
    assert_eq!(curr, 1);
    env.mem.write(lock, 0);
}

fn OSMemoryBarrier(env: &mut Environment) {

}

pub const FUNCTIONS: FunctionExports = &[
    // Trigonometric functions
    export_c_func!(sin(_)),
    export_c_func!(sinf(_)),
    export_c_func!(cos(_)),
    export_c_func!(cosf(_)),
    export_c_func!(tan(_)),
    export_c_func!(tanf(_)),
    export_c_func!(asin(_)),
    export_c_func!(asinf(_)),
    export_c_func!(acos(_)),
    export_c_func!(acosf(_)),
    export_c_func!(atan(_)),
    export_c_func!(atanf(_)),
    export_c_func!(atan2(_, _)),
    export_c_func!(atan2f(_, _)),
    // Hyperbolic functions
    export_c_func!(sinh(_)),
    export_c_func!(sinhf(_)),
    export_c_func!(cosh(_)),
    export_c_func!(coshf(_)),
    export_c_func!(tanh(_)),
    export_c_func!(tanhf(_)),
    export_c_func!(asinh(_)),
    export_c_func!(asinhf(_)),
    export_c_func!(acosh(_)),
    export_c_func!(acoshf(_)),
    export_c_func!(atanh(_)),
    export_c_func!(atanhf(_)),
    // Exponential and logarithmic functions
    export_c_func!(log(_)),
    export_c_func!(logf(_)),
    export_c_func!(log1p(_)),
    export_c_func!(log1pf(_)),
    export_c_func!(log2(_)),
    export_c_func!(log2f(_)),
    export_c_func!(log10(_)),
    export_c_func!(log10f(_)),
    export_c_func!(exp(_)),
    export_c_func!(expf(_)),
    export_c_func!(expm1(_)),
    export_c_func!(expm1f(_)),
    export_c_func!(exp2(_)),
    export_c_func!(exp2f(_)),
    // Power functions
    export_c_func!(pow(_, _)),
    export_c_func!(powf(_, _)),
    export_c_func!(sqrt(_)),
    export_c_func!(sqrtf(_)),
    // Nearest integer functions
    export_c_func!(ceil(_)),
    export_c_func!(ceilf(_)),
    export_c_func!(floor(_)),
    export_c_func!(floorf(_)),
    export_c_func!(round(_)),
    export_c_func!(roundf(_)),
    export_c_func!(trunc(_)),
    export_c_func!(truncf(_)),
    export_c_func!(modff(_, _)),
    // Remainder functions
    export_c_func!(fmod(_, _)),
    export_c_func!(fmodf(_, _)),
    // Maximum, minimum and positive difference functions
    export_c_func!(fmax(_, _)),
    export_c_func!(fmaxf(_, _)),
    export_c_func!(fmin(_, _)),
    export_c_func!(fminf(_, _)),
    export_c_func!(lrintf(_)),
    export_c_func!(__fpclassifyf(_)),
    // Atomic ops (libkern)
    export_c_func!(OSAtomicCompareAndSwap32Barrier(_, _, _)),
    export_c_func!(OSAtomicCompareAndSwapPtr(_, _, _)),
    export_c_func!(OSAtomicAdd32Barrier(_, _)),
    export_c_func!(OSAtomicAdd32(_, _)),
    export_c_func!(OSSpinLockLock(_)),
    export_c_func!(OSSpinLockUnlock(_)),
    export_c_func!(OSMemoryBarrier()),
];
