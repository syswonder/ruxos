/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
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
use ruxtask::AxTaskRef;
use spin::RwLock;

use crate::ctypes;

pub mod condvar;
pub mod mutex;

pub mod futex;

#[cfg(feature = "musl")]
pub mod dummy;
#[cfg(not(feature = "musl"))]
pub mod tsd;
#[cfg(feature = "musl")]
pub use dummy::{
    sys_pthread_getspecific, sys_pthread_key_create, sys_pthread_key_delete,
    sys_pthread_setspecific,
};
#[cfg(not(feature = "musl"))]
pub use tsd::{
    sys_pthread_getspecific, sys_pthread_key_create, sys_pthread_key_delete,
    sys_pthread_setspecific,
};

lazy_static::lazy_static! {
    static ref TID_TO_PTHREAD: RwLock<BTreeMap<u64, ForceSendSync<ctypes::pthread_t>>> = {
        let mut map = BTreeMap::new();
        let main_task = ruxtask::current();
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

        let task_inner = ruxtask::spawn(main);
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
    #[cfg(all(feature = "musl"))]
    fn pcreate(
        _attr: *const ctypes::pthread_attr_t,
        start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
        tls: *mut c_void,
        set_tid: core::sync::atomic::AtomicU64,
        tl: core::sync::atomic::AtomicU64,
    ) -> LinuxResult<(u64, AxTaskRef)> {
        let arg_wrapper = ForceSendSync(arg);

        let my_packet: Arc<Packet<*mut c_void>> = Arc::new(Packet {
            result: UnsafeCell::new(core::ptr::null_mut()),
        });

        let main = move || {
            let arg = arg_wrapper;
            start_routine(arg.0);
        };

        let task_inner = ruxtask::pspawn(main, tls as usize, set_tid, tl);

        let tid = task_inner.id().as_u64();
        let thread = Pthread {
            inner: task_inner.clone(),
            retval: my_packet,
        };
        let ptr = Box::into_raw(Box::new(thread)) as *mut c_void;
        TID_TO_PTHREAD.write().insert(tid, ForceSendSync(ptr));
        Ok((tid, task_inner))
    }

    fn current_ptr() -> *mut Pthread {
        let tid = ruxtask::current().id().as_u64();
        match TID_TO_PTHREAD.read().get(&tid) {
            None => core::ptr::null_mut(),
            Some(ptr) => ptr.0 as *mut Pthread,
        }
    }

    fn current() -> Option<&'static Pthread> {
        unsafe { core::ptr::NonNull::new(Self::current_ptr()).map(|ptr| ptr.as_ref()) }
    }

    #[cfg(feature = "musl")]
    fn exit_musl(_retcode: usize) -> ! {
        let tid = Self::current()
            .expect("fail to get current thread")
            .inner
            .id()
            .as_u64();
        let thread = { TID_TO_PTHREAD.read().get(&tid).unwrap().0 };
        let thread = unsafe { Box::from_raw(thread as *mut Pthread) };
        TID_TO_PTHREAD.write().remove(&tid);
        debug!("Exit_musl, tid: {}", tid);
        drop(thread);
        ruxtask::exit(0)
    }

    #[cfg(not(feature = "musl"))]
    fn exit_current(retval: *mut c_void) -> ! {
        let thread = Self::current().expect("fail to get current thread");
        unsafe { *thread.retval.result.get() = retval };
        ruxtask::exit(0);
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
    #[cfg(feature = "musl")]
    {
        use core::sync::atomic::Ordering;

        let id = ruxtask::current().as_task_ref().id().as_u64();
        // if current task is not `main`
        if id != 2u64 {
            let current = ruxtask::current();
            let current = current.as_task_ref();
            current.free_thread_list_lock();
            let _ = ruxfutex::futex_wake(current.tl().load(Ordering::Relaxed) as usize as _, 1);
        }
        // retval is exit code for musl
        Pthread::exit_musl(retval as usize);
    }
    #[cfg(not(feature = "musl"))]
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
#[cfg(all(feature = "musl"))]
pub unsafe fn sys_clone(
    flags: c_int,
    stack: *mut c_void,
    ptid: *mut ctypes::pid_t,
    tls: *mut c_void,
    ctid: *mut ctypes::pid_t,
) -> c_int {
    debug!(
        "sys_clone <= flags: {:x}, stack: {:p}, ctid: {:x}",
        flags, stack, ctid as usize
    );

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

        let set_tid = if (flags as u32 & ctypes::CLONE_CHILD_SETTID) != 0 {
            core::sync::atomic::AtomicU64::new(ctid as _)
        } else {
            core::sync::atomic::AtomicU64::new(0)
        };

        let (tid, task_inner) = Pthread::pcreate(
            core::ptr::null(),
            func,
            args,
            tls,
            set_tid,
            core::sync::atomic::AtomicU64::from(ctid as u64),
        )?;

        // write tid to ptid
        if (flags as u32 & ctypes::CLONE_PARENT_SETTID) != 0 {
            unsafe { *ptid = tid as c_int };
        }

        ruxtask::put_task(task_inner);

        Ok(tid)
    })
}

/// Create new thread by `sys_clone`, return new thread ID
#[cfg(all(feature = "musl", target_arch = "x86_64"))]
pub unsafe fn sys_clone(
    flags: c_int,
    stack: *mut c_void, // for x86_64, stack points to arg
    ptid: *mut ctypes::pid_t,
    ctid: *mut ctypes::pid_t,
    tls: *mut c_void,
    func: *mut c_void,
) -> c_int {
    debug!(
        "sys_clone <= flags: {:x}, stack: {:p}, ctid: {:x}, func: {:x}, tls: {:#x}",
        flags, stack, ctid as usize, func as usize, tls as usize,
    );

    syscall_body!(sys_clone, {
        if (flags as u32 & ctypes::CLONE_THREAD) == 0 {
            debug!("ONLY support thread");
            return Err(LinuxError::EINVAL);
        }

        let func = unsafe {
            core::mem::transmute::<*const (), extern "C" fn(arg: *mut c_void) -> *mut c_void>(
                func as usize as *const (),
            )
        };
        let args = unsafe { *((stack as usize) as *mut usize) } as *mut c_void;

        let set_tid = if (flags as u32 & ctypes::CLONE_CHILD_SETTID) != 0 {
            core::sync::atomic::AtomicU64::new(ctid as _)
        } else {
            core::sync::atomic::AtomicU64::new(0)
        };

        let (tid, task_inner) = Pthread::pcreate(
            core::ptr::null(),
            func,
            args,
            tls,
            set_tid,
            core::sync::atomic::AtomicU64::from(ctid as u64),
        )?;

        // write tid to ptid
        if (flags as u32 & ctypes::CLONE_PARENT_SETTID) != 0 {
            unsafe { *ptid = tid as c_int };
        }

        ruxtask::put_task(task_inner);

        Ok(tid)
    })
}

/// Set child tid address
#[cfg(feature = "musl")]
pub fn sys_set_tid_address(tid: usize) -> c_int {
    syscall_body!(sys_set_tid_address, {
        debug!("set_tid_address <= addr: {:#x}", tid);
        let id = ruxtask::current().id().as_u64() as c_int;
        ruxtask::current().as_task_ref().set_child_tid(tid);
        Ok(id)
    })
}
