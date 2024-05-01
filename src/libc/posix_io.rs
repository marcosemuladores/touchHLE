/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! POSIX I/O functions (`fcntl.h`, parts of `unistd.h`, etc)

pub mod stat;

use std::cell::{RefCell, RefMut};
use crate::abi::DotDotDot;
use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::{GuestFile, GuestOpenOptions, GuestPath};
use crate::mem::{ConstPtr, ConstVoidPtr, GuestISize, GuestUSize, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::Environment;
use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;
use crate::libc::string::strcat;

#[derive(Default)]
pub struct State {
    /// File descriptors _other than stdin, stdout, and stderr_
    files: Vec<Option<Rc<RefCell<PosixFileHostObject>>>>,
}
impl State {
    pub fn file_for_fd(&mut self, fd: FileDescriptor) -> Option<RefMut<PosixFileHostObject>> {
        self.files
            .get_mut(fd_to_file_idx(fd))
            .and_then(|file_or_none| file_or_none.as_mut())
            .map(|file| file.borrow_mut())
    }
}

pub struct PosixFileHostObject {
    pub file: GuestFile,
    reached_eof: bool,
}

// TODO: stdin/stdout/stderr handling somehow
fn file_idx_to_fd(idx: usize) -> FileDescriptor {
    FileDescriptor::try_from(idx)
        .unwrap()
        .checked_add(NORMAL_FILENO_BASE)
        .unwrap()
}
fn fd_to_file_idx(fd: FileDescriptor) -> usize {
    fd.checked_sub(NORMAL_FILENO_BASE).unwrap() as usize
}

/// File descriptor type. This alias is for readability, POSIX just uses `int`.
pub type FileDescriptor = i32;
pub const STDIN_FILENO: FileDescriptor = 0;
pub const STDOUT_FILENO: FileDescriptor = 1;
pub const STDERR_FILENO: FileDescriptor = 2;
const NORMAL_FILENO_BASE: FileDescriptor = STDERR_FILENO + 1;

/// Flags bitfield for `open`. This alias is for readability, POSIX just uses
/// `int`.
pub type OpenFlag = i32;
pub const O_RDONLY: OpenFlag = 0x0;
pub const O_WRONLY: OpenFlag = 0x1;
pub const O_RDWR: OpenFlag = 0x2;
pub const O_ACCMODE: OpenFlag = O_RDWR | O_WRONLY | O_RDONLY;

pub const O_NONBLOCK: OpenFlag = 0x4;
pub const O_APPEND: OpenFlag = 0x8;
pub const O_SHLOCK: OpenFlag = 0x10;
pub const O_NOFOLLOW: OpenFlag = 0x100;
pub const O_CREAT: OpenFlag = 0x200;
pub const O_TRUNC: OpenFlag = 0x400;
pub const O_EXCL: OpenFlag = 0x800;

pub type FLockFlag = i32;
pub const LOCK_SH: FLockFlag = 1;
#[allow(dead_code)]
pub const LOCK_EX: FLockFlag = 2;
#[allow(dead_code)]
pub const LOCK_NB: FLockFlag = 4;
#[allow(dead_code)]
pub const LOCK_UN: FLockFlag = 8;

pub const F_GETLK: i32 = 7;
pub const F_SETLK: i32 = 8;
pub const F_NOCACHE: i32 = 48;
pub const F_UNLCK: i16 = 2;

#[repr(C, packed)]
#[derive(Debug)]
struct FLockInfo {
    start: off_t,
    len: off_t,
    pid: i32,
    type_: i16,
    whence: i16
}

unsafe impl SafeRead for FLockInfo {}

fn open(env: &mut Environment, path: ConstPtr<u8>, flags: i32, _args: DotDotDot) -> FileDescriptor {
    // TODO: parse variadic arguments and pass them on (file creation mode)
    self::open_direct(env, path, flags)
}

/// Special extension for host code: [open] without the [DotDotDot].
pub fn open_direct(env: &mut Environment, path: ConstPtr<u8>, flags: i32) -> FileDescriptor {
    let res = open_direct2(env, path, flags);
    if res == -1 {
        let buf: MutPtr<u8> = env.mem.alloc(1024).cast();
        _ = env.mem
            .bytes_at_mut(buf, 1024)
            .write(env.bundle.bundle_path().join("").as_str().as_bytes());
        strcat(env, buf, path);
        let res = open_direct2(env, buf.cast_const(), flags);
        env.mem.free(buf.cast());
        return res;
    }
    res
}

/// Special extension for host code: [open] without the [DotDotDot].
fn open_direct2(env: &mut Environment, path: ConstPtr<u8>, flags: i32) -> FileDescriptor {
    // TODO: support more flags, this list is not complete
    assert!(
        flags
            & !(O_ACCMODE
            | O_NONBLOCK
            | O_APPEND
            | O_SHLOCK
            | O_NOFOLLOW
            | O_CREAT
            | O_TRUNC
            | O_EXCL)
            == 0
    );
    // TODO: symlinks don't exist in the FS yet, so we can't "not follow" them.
    // (Should we just ignore this?)
    assert!(flags & O_NOFOLLOW == 0);
    // TODO: exclusive mode not implemented yet
    assert!(flags & O_EXCL == 0);

    if path.is_null() {
        log_dbg!("open({:?}, {:#x}) => -1", path, flags);
        return -1; // TODO: set errno to EFAULT
    }

    // TODO: respect the mode (in the variadic arguments) when creating a file
    // Note: NONBLOCK flag is ignored, assumption is all file I/O is fast
    let mut options = GuestOpenOptions::new();
    match flags & O_ACCMODE {
        O_RDONLY => options.read(),
        O_WRONLY => options.write(),
        O_RDWR => options.read().write(),
        _ => panic!(),
    };
    if (flags & O_APPEND) != 0 {
        options.append();
    }
    if (flags & O_CREAT) != 0 {
        options.create();
    }
    if (flags & O_TRUNC) != 0 {
        options.truncate();
    }

    let path_string = match env.mem.cstr_at_utf8(path) {
    let y = env.mem.cstr_at_utf8(path);
        Ok(path_str) => path_str.to_owned(),
    if y.is_err() {
        Err(err) => {
        return -1;
            log!(
                "open() error, unable to treat {:?} as utf8 str: {:?}",
                path,
                err
            );
            // TODO: set errno
            return -1;
        }
    };
    // TODO: symlinks don't exist in the FS yet, so we can't "not follow" them.
    if flags & O_NOFOLLOW != 0 {
        log!("Ignoring O_NOFOLLOW when opening {:?}", path_string);
    }
    let path_string = y.unwrap().to_owned();
    let res = match env
        .fs
        .open_with_options(GuestPath::new(&path_string), options)
    {
        Ok(file) => {
            let host_object = PosixFileHostObject {
                file,
                reached_eof: false,
            };

            let idx = if let Some(free_idx) = env
                .libc_state
                .posix_io
                .files
                .iter()
                .position(|f| f.is_none())
            {
                env.libc_state.posix_io.files[free_idx] = Some(Rc::new(RefCell::new(host_object)));
                free_idx
            } else {
                let idx = env.libc_state.posix_io.files.len();
                env.libc_state.posix_io.files.push(Some(Rc::new(RefCell::new(host_object))));
                idx
            };
            file_idx_to_fd(idx)
        }
        Err(()) => {
            // TODO: set errno
            -1
        }
    };
    if res != -1 && (flags & O_SHLOCK) != 0 {
        // TODO: Handle possible errors
        flock(env, res, LOCK_SH);
    }
    log_dbg!(
        "open({:?} {:?}, {:#x}) => {:?}",
        path,
        path_string,
        flags,
        res
    );
    res
}

fn dup(env: &mut Environment, fd: FileDescriptor) -> FileDescriptor {
    let Some(file) = env.libc_state.posix_io.files[fd_to_file_idx(fd)].as_ref() else {
        return -1; // TODO: set errno
    };

    let idx = if let Some(free_idx) = env
        .libc_state
        .posix_io
        .files
        .iter()
        .position(|f| f.is_none())
    {
        env.libc_state.posix_io.files[free_idx] = Some(file.clone());
        free_idx
    } else {
        let idx = env.libc_state.posix_io.files.len();
        env.libc_state.posix_io.files.push(Some(file.clone()));
        idx
    };
    file_idx_to_fd(idx)
}

pub fn read(
    env: &mut Environment,
    fd: FileDescriptor,
    buffer: MutVoidPtr,
    size: GuestUSize,
) -> GuestISize {
    if buffer.is_null() {
        return 0;
    }
    // TODO: error handling for unknown fd?
    let mut file = env.libc_state.posix_io.file_for_fd(fd).unwrap();

    let buffer_slice = env.mem.bytes_at_mut(buffer.cast(), size);
    match file.file.read(buffer_slice) {
        Ok(bytes_read) => {
            if bytes_read == 0 && size != 0 {
                // need to set EOF
                file.reached_eof = true;
            }
            if bytes_read < buffer_slice.len() {
                log!(
                    "Warning: read({:?}, {:?}, {:#x}) read only {:#x} bytes",
                    fd,
                    buffer,
                    size,
                    bytes_read,
                );
            } else {
                log_dbg!(
                    "read({:?}, {:?}, {:#x}) => {:#x}",
                    fd,
                    buffer,
                    size,
                    bytes_read,
                );
            }
            bytes_read.try_into().unwrap()
        }
        Err(e) => {
            // TODO: set errno
            log!(
                "Warning: read({:?}, {:?}, {:#x}) encountered error {:?}, returning -1",
                fd,
                buffer,
                size,
                e,
            );
            -1
        }
    }
}

pub fn pread(
    env: &mut Environment,
    fd: FileDescriptor,
    buffer: MutVoidPtr,
    size: GuestUSize,
    offset: off_t,
) -> GuestISize {
    let old = lseek(env, fd, 0, SEEK_CUR);
    lseek(env, fd, offset, SEEK_SET);
    let ret = read(env, fd, buffer, size);
    lseek(env, fd, old, SEEK_SET);
    ret
}

pub fn pwrite(
    env: &mut Environment,
    fd: FileDescriptor,
    buffer: ConstVoidPtr,
    size: GuestUSize,
    offset: off_t,
) -> GuestISize {
    let old = lseek(env, fd, 0, SEEK_CUR);
    lseek(env, fd, offset, SEEK_SET);
    let ret = write(env, fd, buffer, size);
    lseek(env, fd, old, SEEK_SET);
    ret
}

/// Helper for C `feof()`.
pub(super) fn eof(env: &mut Environment, fd: FileDescriptor) -> i32 {
    let file = env.libc_state.posix_io.file_for_fd(fd).unwrap();
    if file.reached_eof {
        1
    } else {
        0
    }
}

pub(super) fn clearerr(env: &mut Environment, fd: FileDescriptor) {
    let mut file = env.libc_state.posix_io.file_for_fd(fd).unwrap();
    file.reached_eof = false;
}


pub fn write(
    env: &mut Environment,
    fd: FileDescriptor,
    buffer: ConstVoidPtr,
    size: GuestUSize,
) -> GuestISize {
    if fd == STDERR_FILENO {
        let buffer_slice = env.mem.bytes_at(buffer.cast(), size);
        return match std::io::stderr().write(buffer_slice) {
            Ok(bytes_written) => bytes_written as GuestUSize,
            Err(_err) => 0,
        } as GuestISize
    }
    if fd == STDOUT_FILENO {
        let buffer_slice = env.mem.bytes_at(buffer.cast(), size);
        return match std::io::stdout().write(buffer_slice) {
            Ok(bytes_written) => bytes_written as GuestUSize,
            Err(_err) => 0,
        } as GuestISize
    }
    // TODO: error handling for unknown fd?
    // if env.libc_state.posix_io.file_for_fd(fd).is_none() {
    //     return -1;
    // }
    let mut file = env.libc_state.posix_io.file_for_fd(fd).unwrap();

    let buffer_slice = env.mem.bytes_at(buffer.cast(), size);
    match file.file.write(buffer_slice) {
        Ok(bytes_written) => {
            if bytes_written < buffer_slice.len() {
                log!(
                    "Warning: write({:?}, {:?}, {:#x}) wrote only {:#x} bytes",
                    fd,
                    buffer,
                    size,
                    bytes_written,
                );
            } else {
                log_dbg!(
                    "write({:?}, {:?}, {:#x}) => {:#x}",
                    fd,
                    buffer,
                    size,
                    bytes_written,
                );
            }
            bytes_written.try_into().unwrap()
        }
        Err(e) => {
            // TODO: set errno
            log!(
                "Warning: write({:?}, {:?}, {:#x}) encountered error {:?}, returning -1",
                fd,
                buffer,
                size,
                e,
            );
            -1
        }
    }
}

#[allow(non_camel_case_types)]
pub type off_t = i64;
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;
pub fn lseek(env: &mut Environment, fd: FileDescriptor, offset: off_t, whence: i32) -> off_t {
    // TODO: error handling for unknown fd?
    let mut file = env.libc_state.posix_io.file_for_fd(fd).unwrap();

    let from = match whence {
        // not sure whether offset is treated as signed or unsigned when using
        // SEEK_SET, so `.try_into()` seems safer.
        SEEK_SET => SeekFrom::Start(offset.try_into().unwrap()),
        SEEK_CUR => SeekFrom::Current(offset),
        SEEK_END => SeekFrom::End(offset),
        _ => panic!("Unsupported \"whence\" parameter to seek(): {}", whence),
    };

    let res = match file.file.seek(from) {
        Ok(new_offset) => {
            // "A successful call to the fseek() function clears
            // the end-of-file indicator for the stream..."
            file.reached_eof = false;

            new_offset.try_into().unwrap()
        }
        // TODO: set errno
        Err(_) => -1,
    };
    log_dbg!("lseek({:?}, {:#x}, {}) => {}", fd, offset, whence, res);
    res
}

pub fn close(env: &mut Environment, fd: FileDescriptor) -> i32 {
    // TODO: error handling for unknown fd?
    if fd < 0 || matches!(fd, STDOUT_FILENO | STDERR_FILENO) {
        return 0;
    }

    match env.libc_state.posix_io.files[fd_to_file_idx(fd)].take() {
        Some(file) => {
            // The actual closing of the file happens implicitly when `file` falls out
            // of scope. The return value is about whether flushing succeeds.
            match Rc::into_inner(file).map(|f| f.into_inner().file.sync_all()) {
                Some(Ok(())) => {
                    log_dbg!("close({:?}) => 0", fd);
                    0
                }
                Some(Err(_)) => {
                    // TODO: set errno
                    log!("Warning: close({:?}) failed, returning -1", fd);
                    -1
                },
                None => {
                    log_dbg!("close({:?}) => 0, references remaining", fd);
                    0
                }
            }
        }
        None => {
            // TODO: set errno
            log!("Warning: close({:?}) failed, returning -1", fd);
            -1
        }
    }
}

pub fn getcwd(env: &mut Environment, buf_ptr: MutPtr<u8>, buf_size: GuestUSize) -> MutPtr<u8> {
    let working_directory = env.fs.working_directory();
    if !env.fs.is_dir(working_directory) {
        // TODO: set errno to ENOENT
        log!(
            "Warning: getcwd({:?}, {:#x}) failed, returning NULL",
            buf_ptr,
            buf_size
        );
        return Ptr::null();
    }

    let working_directory = env.fs.working_directory().as_str().as_bytes();

    if buf_ptr.is_null() {
        // The buffer size argument is presumably ignored in this mode.
        // This mode is an extension, which might explain the strange API.
        let res = env.mem.alloc_and_write_cstr(working_directory);
        log_dbg!("getcwd(NULL, _) => {:?} ({:?})", res, working_directory);
        return res;
    }

    // Includes space for null terminator
    let res_size: GuestUSize = u32::try_from(working_directory.len()).unwrap() + 1;

    if buf_size < res_size {
        // TODO: set errno to EINVAL or ERANGE as appropriate
        log!(
            "Warning: getcwd({:?}, {:#x}) failed, returning NULL",
            buf_ptr,
            buf_size
        );
        return Ptr::null();
    }

    let buf = env.mem.bytes_at_mut(buf_ptr, res_size);
    buf[..(res_size - 1) as usize].copy_from_slice(working_directory);
    buf[(res_size - 1) as usize] = b'\0';

    log_dbg!(
        "getcwd({:?}, {:#x}) => {:?}, wrote {:?} ({:#x} bytes)",
        buf_ptr,
        buf_size,
        buf_ptr,
        working_directory,
        res_size
    );
    buf_ptr
}

fn chdir(env: &mut Environment, path_ptr: ConstPtr<u8>) -> i32 {
    let path = GuestPath::new(env.mem.cstr_at_utf8(path_ptr).unwrap());
    match env.fs.change_working_directory(path) {
        Ok(new) => {
            log_dbg!(
                "chdir({:?}) => 0, new working directory: {:?}",
                path_ptr,
                new,
            );
            0
        }
        Err(()) => {
            log!("Warning: chdir({:?}) failed, could not change working directory to {:?}, returning -1", path_ptr, path);
            // TODO: set errno
            -1
        }
    }
}
// TODO: fchdir(), once open() on a directory is supported.

fn flock(_env: &mut Environment, fd: FileDescriptor, operation: FLockFlag) -> i32 {
    log!("TODO: flock({:?}, {:?})", fd, operation);
    0
}

fn ftruncate(env: &mut Environment, fd: FileDescriptor, len: off_t) -> i32 {
    let file = env.libc_state.posix_io.file_for_fd(fd).unwrap();
    match file.file.set_len(len as u64) {
        Ok(()) => 0,
        Err(_) => -1, // TODO: set errno
    }
}

fn fcntl(env: &mut Environment, fd: FileDescriptor, operation: i32, args: DotDotDot) -> i32 {
    match operation {
        F_GETLK => {
            let ptr = args.start().next::<MutPtr<FLockInfo>>(env);
            let mut data = env.mem.read(ptr);
            data.type_ = F_UNLCK;
            env.mem.write(ptr, data);
            0
        },
        F_SETLK => {
            let ptr = args.start().next::<MutPtr<FLockInfo>>(env);
            let data = env.mem.read(ptr);
            log!("TODO: fcntl({:?}, {:?}, {:?})", fd, operation, data);
            0
        }
        F_NOCACHE => {
            log!("Ignoring F_NOCACHE for {} fd", fd);
            0
        }
        _ => unimplemented!("fcntl({}, {})", fd, operation)
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(open(_, _, _)),
    export_c_func!(read(_, _, _)),
    export_c_func!(dup(_)),
    export_c_func!(pread(_, _, _, _)),
    export_c_func!(write(_, _, _)),
    export_c_func!(pwrite(_, _, _, _)),
    export_c_func!(lseek(_, _, _)),
    export_c_func!(close(_)),
    export_c_func!(getcwd(_, _)),
    export_c_func!(chdir(_)),
    export_c_func!(flock(_, _)),
    export_c_func!(ftruncate(_, _)),
    export_c_func!(fcntl(_, _, _)),
];
