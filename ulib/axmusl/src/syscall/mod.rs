pub mod syscall_id;

use arceos_posix_api::ctypes;
use core::ffi::c_int;
use syscall_id::SyscallId;

pub fn syscall(syscall_id: SyscallId, args: [usize; 6]) -> isize {
    debug!("syscall <= syscall_name: {:?}", syscall_id);

    unsafe {
        match syscall_id {
            SyscallId::INVALID => arceos_posix_api::sys_invalid(syscall_id as usize as c_int) as _,
            #[cfg(feature = "fd")]
            SyscallId::IOCTL => {
                arceos_posix_api::sys_ioctl(args[0] as c_int, args[1], args[2]) as _
            }
            #[cfg(feature = "fs")]
            SyscallId::OPENAT => arceos_posix_api::sys_openat(
                args[0],
                args[1] as *const core::ffi::c_char,
                args[2] as c_int,
                args[3] as ctypes::mode_t,
            ) as _,
            SyscallId::READ => arceos_posix_api::sys_read(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2],
            ) as _,
            SyscallId::WRITE => arceos_posix_api::sys_write(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2],
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::CLOSE => arceos_posix_api::sys_close(args[0] as c_int) as _,
            // #[cfg(feature = "fs")]
            // SyscallId::STAT => arceos_posix_api::sys_stat(
            //     args[0] as *const core::ffi::c_char,
            //     args[1] as *mut ctypes::stat,
            // ) as _,
            #[cfg(feature = "fs")]
            SyscallId::FSTAT => {
                arceos_posix_api::sys_fstat(args[0] as c_int, args[1] as *mut ctypes::stat) as _
            }
            // #[cfg(feature = "fs")]
            // SyscallId::LSTAT => arceos_posix_api::sys_lstat(
            //     args[0] as *const core::ffi::c_char,
            //     args[1] as *mut ctypes::stat,
            // ) as _,
            #[cfg(feature = "fs")]
            SyscallId::LSEEK => arceos_posix_api::sys_lseek(
                args[0] as c_int,
                args[1] as ctypes::off_t,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::WRITEV => arceos_posix_api::sys_writev(
                args[0] as c_int,
                args[1] as *const ctypes::iovec,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "pipe")]
            SyscallId::PIPE2 => arceos_posix_api::sys_pipe2(
                core::slice::from_raw_parts_mut(args[0] as *mut c_int, 2),
                args[1] as c_int,
            ) as _,
            #[cfg(feature = "multitask")]
            SyscallId::SET_TID_ADDRESS => arceos_posix_api::sys_set_tid_address(args[0]) as _,
            // #[cfg(feature = "select")]
            // SyscallId::SELECT => arceos_posix_api::sys_select(
            //     args[0] as c_int,
            //     args[1] as *mut ctypes::fd_set,
            //     args[2] as *mut ctypes::fd_set,
            //     args[3] as *mut ctypes::fd_set,
            //     args[4] as *mut ctypes::timeval,
            // ) as _,
            SyscallId::SCHED_YIELD => arceos_posix_api::sys_sched_yield() as _,
            #[cfg(feature = "fd")]
            SyscallId::DUP => arceos_posix_api::sys_dup(args[0] as c_int) as _,
            // #[cfg(feature = "fd")]
            // SyscallId::DUP2 => arceos_posix_api::sys_dup2(args[0] as _, args[1] as _) as _,
            SyscallId::NANO_SLEEP => arceos_posix_api::sys_nanosleep(
                args[0] as *const ctypes::timespec,
                args[1] as *mut ctypes::timespec,
            ) as _,
            #[cfg(feature = "multitask")]
            SyscallId::GETPID => arceos_posix_api::sys_getpid() as _,
            #[cfg(feature = "net")]
            SyscallId::SOCKET => {
                arceos_posix_api::sys_socket(args[0] as c_int, args[1] as c_int, args[2] as c_int)
                    as _
            }
            #[cfg(feature = "net")]
            SyscallId::CONNECT => arceos_posix_api::sys_connect(
                args[0] as c_int,
                args[1] as *const ctypes::sockaddr,
                args[2] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::ACCEPT => arceos_posix_api::sys_accept(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::SENDTO => arceos_posix_api::sys_sendto(
                args[0] as c_int,
                args[1] as *const core::ffi::c_void,
                args[2] as ctypes::size_t,
                args[3] as c_int,
                args[4] as *const ctypes::sockaddr,
                args[5] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::RECVFROM => arceos_posix_api::sys_recvfrom(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2] as ctypes::size_t,
                args[3] as c_int,
                args[4] as *mut ctypes::sockaddr,
                args[5] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::SHUTDOWN => {
                arceos_posix_api::sys_shutdown(args[0] as c_int, args[1] as c_int) as _
            }
            #[cfg(feature = "net")]
            SyscallId::BIND => arceos_posix_api::sys_bind(
                args[0] as c_int,
                args[1] as *const ctypes::sockaddr,
                args[2] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::LISTEN => {
                arceos_posix_api::sys_listen(args[0] as c_int, args[1] as c_int) as _
            }
            #[cfg(feature = "net")]
            SyscallId::GETSOCKNAME => arceos_posix_api::sys_getsockname(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::GETPEERNAME => arceos_posix_api::sys_getpeername(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[allow(unreachable_code)]
            #[cfg(not(feature = "multitask"))]
            SyscallId::EXIT => arceos_posix_api::sys_exit(args[0] as c_int) as _,
            #[allow(unreachable_code)]
            #[cfg(feature = "multitask")]
            SyscallId::EXIT => {
                arceos_posix_api::sys_pthread_exit(args[0] as *mut core::ffi::c_void) as _
            }
            #[cfg(feature = "fd")]
            SyscallId::FCNTL => {
                arceos_posix_api::sys_fcntl(args[0] as c_int, args[1] as c_int, args[2]) as _
            }
            #[cfg(feature = "fs")]
            SyscallId::GETCWD => {
                arceos_posix_api::sys_getcwd(args[0] as *mut core::ffi::c_char, args[1]) as _
            }
            // #[cfg(feature = "fs")]
            // SyscallId::RENAME => arceos_posix_api::sys_rename(
            //     args[0] as *const core::ffi::c_char,
            //     args[1] as *const core::ffi::c_char,
            // ) as _,
            // #[cfg(feature = "epoll")]
            // SyscallId::EPOLL_CREATE => arceos_posix_api::sys_epoll_create(args[0] as c_int) as _,
            SyscallId::CLOCK_GETTIME => arceos_posix_api::sys_clock_gettime(
                args[0] as ctypes::clockid_t,
                args[1] as *mut ctypes::timespec,
            ) as _,
            // #[cfg(feature = "epoll")]
            // SyscallId::EPOLL_WAIT => crate::sys_epoll_wait(
            //     args[0] as c_int,
            //     args[1] as *mut ctypes::epoll_event,
            //     args[2] as c_int,
            //     args[3] as c_int,
            // ) as _,
            #[cfg(feature = "epoll")]
            SyscallId::EPOLL_CTL => arceos_posix_api::sys_epoll_ctl(
                args[0] as c_int,
                args[1] as c_int,
                args[2] as c_int,
                args[3] as *mut ctypes::epoll_event,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::RT_SIGPROCMASK => arceos_posix_api::sys_rt_sigprocmask(
                args[0] as c_int,
                args[1] as *const usize,
                args[2] as *mut usize,
                args[3],
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MUNMAP => arceos_posix_api::sys_munmap(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "multitask")]
            SyscallId::CLONE => arceos_posix_api::sys_clone(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2] as *mut ctypes::pid_t,
                args[3] as *mut core::ffi::c_void,
                args[4] as *mut ctypes::pid_t,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MMAP => arceos_posix_api::sys_mmap(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
                args[3] as c_int,
                args[4] as c_int,
                args[5] as ctypes::off_t,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MPROTECT => arceos_posix_api::sys_mprotect(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "multitask")]
            SyscallId::FUTEX => arceos_posix_api::sys_futex(
                args[0],
                args[1] as c_int,
                args[2] as c_int,
                args[3],
                args[4] as c_int,
                args[5] as c_int,
            ) as _,
            // #[cfg(feature = "fd")]
            // SyscallId::DUP3 => {
            //     crate::sys_dup3(args[0] as c_int, args[1] as c_int, args[2] as c_int) as _
            // }

            // #[cfg(feature = "net")]
            // SyscallId::SEND => crate::sys_send(
            //     args[0] as c_int,
            //     args[1] as *const core::ffi::c_void,
            //     args[2] as ctypes::size_t,
            //     args[3] as c_int,
            // ) as _,

            // #[cfg(feature = "net")]
            // SyscallId::RECV => crate::sys_recv(
            //     args[0] as c_int,
            //     args[1] as *mut core::ffi::c_void,
            //     args[2] as ctypes::size_t,
            //     args[3] as c_int,
            // ) as _,
            // #[cfg(feature = "net")]
            // SyscallId::GETADDRINFO => crate::sys_getaddrinfo(
            //     args[0] as *const core::ffi::c_char,
            //     args[1] as *const core::ffi::c_char,
            //     args[2] as *mut ctypes::sockaddr,
            //     args[3] as ctypes::size_t,
            // ) as _,
            // #[cfg(feature = "fs")]
            // SyscallId::OPEN => crate::sys_open(
            //     args[0] as *const core::ffi::c_char,
            //     args[1] as c_int,
            //     args[2] as ctypes::mode_t,
            // ) as _,
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_SELF => crate::sys_pthread_self() as _,
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_CREATE => crate::sys_pthread_create(
            //     args[0] as *mut ctypes::pthread_t,
            //     args[1] as *const ctypes::pthread_attr_t,
            //     args[2] as *mut core::ffi::c_void,
            //     args[3] as *mut core::ffi::c_void,
            // ) as _,
            // #[allow(unreachable_code)]
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_EXIT => {
            //     crate::sys_pthread_exit(args[0] as *mut core::ffi::c_void) as _
            // }
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_JOIN => crate::sys_pthread_join(
            //     args[0] as ctypes::pthread_t,
            //     args[1] as *mut *mut core::ffi::c_void,
            // ) as _,
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_MUTEX_INIT => crate::sys_pthread_mutex_init(
            //     args[0] as *mut ctypes::pthread_mutex_t,
            //     args[1] as *const ctypes::pthread_mutexattr_t,
            // ) as _,
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_MUTEX_LOCK => {
            //     crate::sys_pthread_mutex_lock(args[0] as *mut ctypes::pthread_mutex_t) as _
            // }
            // #[cfg(feature = "multitask")]
            // SyscallId::PTHREAD_MUTEX_UNLOCK => {
            //     crate::sys_pthread_mutex_unlock(args[0] as *mut ctypes::pthread_mutex_t) as _
            // }
        }
    }
}
