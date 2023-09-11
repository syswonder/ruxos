/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::cell::UnsafeCell;
use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use axtask::AxTaskRef;
use spin::RwLock;

use crate::ctypes;

pub mod condvar;
pub mod futex;
pub mod mutex;

lazy_static::lazy_static! {
    static ref TID_TO_PTHREAD: RwLock<BTreeMap<u64, ForceSendSync<ctypes::pthread_t>>> = {
        let mut map = BTreeMap::new();
        let main_task = axtask::current();
        let main_tid = main_task.id().as_u64();
        let main_thread = Pthread {
            inner: main_task.as_task_ref().clone(),
            retval: Arc::new(Packet {
                result: UnsafeCell::new(core::ptr::null_mut()),
            }),
        };
        let ptr = Box::into_raw(Box::new(main_thread)) as *mut c_void;
        map.insert(main_tid, ForceSendSync(ptr));
        RwLock::new(map)
    };
}

struct Packet<T> {
    result: UnsafeCell<T>,
}

unsafe impl<T> Send for Packet<T> {}
unsafe impl<T> Sync for Packet<T> {}

pub struct Pthread {
    inner: AxTaskRef,
    retval: Arc<Packet<*mut c_void>>,
}

impl Pthread {
    fn create(
        _attr: *const ctypes::pthread_attr_t,
        start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> LinuxResult<ctypes::pthread_t> {
        let arg_wrapper = ForceSendSync(arg);

        let my_packet: Arc<Packet<*mut c_void>> = Arc::new(Packet {
            result: UnsafeCell::new(core::ptr::null_mut()),
        });
        let their_packet = my_packet.clone();

        let main = move || {
            let arg = arg_wrapper;
            let ret = start_routine(arg.0);
            unsafe { *their_packet.result.get() = ret };
            drop(their_packet);
        };

        let task_inner = axtask::spawn(main);
        let tid = task_inner.id().as_u64();
        let thread = Pthread {
            inner: task_inner,
            retval: my_packet,
        };
        let ptr = Box::into_raw(Box::new(thread)) as *mut c_void;
        TID_TO_PTHREAD.write().insert(tid, ForceSendSync(ptr));
        Ok(ptr)
    }

    /// Posix create, used by musl libc
    fn pcreate(
        _attr: *const ctypes::pthread_attr_t,
        start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
        tls: *mut c_void,
        tl: Option<usize>,
    ) -> LinuxResult<u64> {
        let arg_wrapper = ForceSendSync(arg);

        let my_packet: Arc<Packet<*mut c_void>> = Arc::new(Packet {
            result: UnsafeCell::new(core::ptr::null_mut()),
        });

        let main = move || {
            let arg = arg_wrapper;
            start_routine(arg.0);
        };

        let task_inner = axtask::spawn_musl(main, tls as usize, tl);

        let tid = task_inner.id().as_u64();
        let thread = Pthread {
            inner: task_inner,
            retval: my_packet,
        };
        let ptr = Box::into_raw(Box::new(thread)) as *mut c_void;
        TID_TO_PTHREAD.write().insert(tid, ForceSendSync(ptr));
        Ok(tid)
    }

    fn current_ptr() -> *mut Pthread {
        let tid = axtask::current().id().as_u64();
        match TID_TO_PTHREAD.read().get(&tid) {
            None => core::ptr::null_mut(),
            Some(ptr) => ptr.0 as *mut Pthread,
        }
    }

    fn current() -> Option<&'static Pthread> {
        unsafe { core::ptr::NonNull::new(Self::current_ptr()).map(|ptr| ptr.as_ref()) }
    }

    fn exit_current(retval: *mut c_void) -> ! {
        {
            let thread = Self::current().expect("fail to get current thread");
            unsafe { *thread.retval.result.get() = retval };
        }

        #[cfg(feature = "musl")]
        {
            let tid = axtask::current().id().as_u64();
            let thread = { TID_TO_PTHREAD.read().get(&tid).unwrap().0 };
            let thread = unsafe { Box::from_raw(thread as *mut Pthread) };

            TID_TO_PTHREAD.write().remove(&tid);
            drop(thread);
        }

        axtask::exit(0);
    }

    fn join(ptr: ctypes::pthread_t) -> LinuxResult<*mut c_void> {
        if core::ptr::eq(ptr, Self::current_ptr() as _) {
            return Err(LinuxError::EDEADLK);
        }

        let thread = unsafe { Box::from_raw(ptr as *mut Pthread) };
        thread.inner.join();
        let tid = thread.inner.id().as_u64();
        let retval = unsafe { *thread.retval.result.get() };
        TID_TO_PTHREAD.write().remove(&tid);
        drop(thread);
        Ok(retval)
    }
}

/// Returns the `pthread` struct of current thread.
pub fn sys_pthread_self() -> ctypes::pthread_t {
    Pthread::current().expect("fail to get current thread") as *const Pthread as _
}

/// Create a new thread with the given entry point and argument.
///
/// If successful, it stores the pointer to the newly created `struct __pthread`
/// in `res` and returns 0.
pub unsafe fn sys_pthread_create(
    res: *mut ctypes::pthread_t,
    attr: *const ctypes::pthread_attr_t,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    debug!(
        "sys_pthread_create <= {:#x}, {:#x}",
        start_routine as usize, arg as usize
    );
    syscall_body!(sys_pthread_create, {
        let ptr = Pthread::create(attr, start_routine, arg)?;
        unsafe { core::ptr::write(res, ptr) };
        Ok(0)
    })
}

/// Exits the current thread. The value `retval` will be returned to the joiner.
pub fn sys_pthread_exit(retval: *mut c_void) -> ! {
    debug!("sys_pthread_exit <= {:#x}", retval as usize);
    let id = axtask::current().as_task_ref().id().as_u64();
    if id != 2u64 {
        axtask::current().as_task_ref().free_thread_list_lock();
    }
    // retval is exit code for musl
    Pthread::exit_current(retval);
}

/// Waits for the given thread to exit, and stores the return value in `retval`.
pub unsafe fn sys_pthread_join(thread: ctypes::pthread_t, retval: *mut *mut c_void) -> c_int {
    debug!("sys_pthread_join <= {:#x}", retval as usize);
    syscall_body!(sys_pthread_join, {
        let ret = Pthread::join(thread)?;
        if !retval.is_null() {
            unsafe { core::ptr::write(retval, ret) };
        }
        Ok(0)
    })
}

#[derive(Clone, Copy)]
struct ForceSendSync<T>(T);

unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}

/// Create new thread by `sys_clone`, return new thread ID
pub unsafe fn sys_clone(
    flags: c_int,
    stack: *mut c_void,
    ptid: *mut ctypes::pid_t,
    tls: *mut c_void,
    ctid: *mut ctypes::pid_t,
) -> c_int {
    debug!("sys_clone <= flags: {:x}, stack: {:p}", flags, stack);

    syscall_body!(sys_clone, {
        if (flags as u32 & ctypes::CLONE_THREAD) == 0 {
            debug!("ONLY support thread");
            return Err(LinuxError::EINVAL);
        }

        let func = unsafe {
            core::mem::transmute::<*const (), extern "C" fn(arg: *mut c_void) -> *mut c_void>(
                (*(stack as *mut usize)) as *const (),
            )
        };
        let args = unsafe { *((stack as usize + 8) as *mut usize) } as *mut c_void;

        let clear_tid = if (flags as u32 & ctypes::CLONE_CHILD_CLEARTID) != 0 {
            Some(ctid as usize)
        } else {
            None
        };
        let tid = Pthread::pcreate(core::ptr::null(), func, args, tls, clear_tid)?;

        // write tid to ptid
        if (flags as u32 & ctypes::CLONE_PARENT_SETTID) != 0 {
            unsafe { *ptid = tid as c_int };
        }

        // clear tid in ctid
        if (flags as u32 & ctypes::CLONE_CHILD_CLEARTID) != 0 {
            unsafe { *ctid = 0 as _ };
        }

        Ok(tid)
    })
}

/// Set child tid address
pub fn sys_set_tid_address(addr: usize) -> c_int {
    debug!("set_tid_address <= addr: {:#x}", addr);
    let id = axtask::current().id().as_u64() as c_int;
    unsafe {
        (addr as *mut c_int).write_volatile(id);
    }
    id
}
