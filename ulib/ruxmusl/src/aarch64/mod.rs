pub mod syscall_id;

use core::ffi::c_int;
use ruxos_posix_api::ctypes;
use syscall_id::SyscallId;

pub fn syscall(syscall_id: SyscallId, args: [usize; 6]) -> isize {
    debug!("syscall <= syscall_name: {:?}", syscall_id);

    unsafe {
        match syscall_id {
            SyscallId::INVALID => ruxos_posix_api::sys_invalid(syscall_id as usize as c_int) as _,
            #[cfg(feature = "fs")]
            SyscallId::GETCWD => {
                ruxos_posix_api::sys_getcwd(args[0] as *mut core::ffi::c_char, args[1]) as _
            }
            #[cfg(feature = "epoll")]
            SyscallId::EPOLL_CREATE1 => ruxos_posix_api::sys_epoll_create(args[0] as c_int) as _,
            #[cfg(feature = "epoll")]
            SyscallId::EPOLL_CTL => ruxos_posix_api::sys_epoll_ctl(
                args[0] as c_int,
                args[1] as c_int,
                args[2] as c_int,
                args[3] as *mut ctypes::epoll_event,
            ) as _,
            #[cfg(feature = "epoll")]
            SyscallId::EPOLL_PWAIT => ruxos_posix_api::sys_epoll_pwait(
                args[0] as c_int,
                args[1] as *mut ctypes::epoll_event,
                args[2] as c_int,
                args[3] as c_int,
                args[4] as *const ctypes::sigset_t,
                args[5] as *const ctypes::size_t,
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::DUP => ruxos_posix_api::sys_dup(args[0] as c_int) as _,
            #[cfg(feature = "fd")]
            SyscallId::DUP3 => {
                ruxos_posix_api::sys_dup3(args[0] as c_int, args[1] as c_int, args[2] as c_int) as _
            }
            #[cfg(feature = "fd")]
            SyscallId::FCNTL => {
                ruxos_posix_api::sys_fcntl(args[0] as c_int, args[1] as c_int, args[2]) as _
            }
            #[cfg(feature = "fd")]
            SyscallId::IOCTL => ruxos_posix_api::sys_ioctl(args[0] as c_int, args[1], args[2]) as _,
            #[cfg(feature = "fs")]
            SyscallId::MKDIRAT => ruxos_posix_api::sys_mkdirat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as ctypes::mode_t,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::UNLINKAT => ruxos_posix_api::sys_unlinkat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::FCHOWNAT => ruxos_posix_api::sys_fchownat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as ctypes::uid_t,
                args[3] as ctypes::gid_t,
                args[4] as c_int,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::RENAMEAT => ruxos_posix_api::sys_renameat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as c_int,
                args[3] as *const core::ffi::c_char,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::OPENAT => ruxos_posix_api::sys_openat(
                args[0],
                args[1] as *const core::ffi::c_char,
                args[2] as c_int,
                args[3] as ctypes::mode_t,
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::CLOSE => ruxos_posix_api::sys_close(args[0] as c_int) as _,
            #[cfg(feature = "pipe")]
            SyscallId::PIPE2 => ruxos_posix_api::sys_pipe2(
                core::slice::from_raw_parts_mut(args[0] as *mut c_int, 2),
                args[1] as c_int,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::GETDENTS64 => ruxos_posix_api::sys_getdents64(
                args[0] as c_int,
                args[1] as *mut ctypes::dirent,
                args[2] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::LSEEK => ruxos_posix_api::sys_lseek(
                args[0] as c_int,
                args[1] as ctypes::off_t,
                args[2] as c_int,
            ) as _,
            SyscallId::READ => ruxos_posix_api::sys_read(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2],
            ) as _,
            SyscallId::WRITE => ruxos_posix_api::sys_write(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2],
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::READV => ruxos_posix_api::sys_readv(
                args[0] as c_int,
                args[1] as *const ctypes::iovec,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "fd")]
            SyscallId::WRITEV => ruxos_posix_api::sys_writev(
                args[0] as c_int,
                args[1] as *const ctypes::iovec,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::PREAD64 => ruxos_posix_api::sys_pread(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2] as ctypes::size_t,
                args[3] as ctypes::off_t,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::PREADV => ruxos_posix_api::sys_preadv(
                args[0] as c_int,
                args[1] as *const ctypes::iovec,
                args[2] as c_int,
                args[3] as ctypes::off_t,
            ) as _,
            #[cfg(feature = "select")]
            SyscallId::PSELECT6 => ruxos_posix_api::sys_pselect6(
                args[0] as c_int,
                args[1] as *mut ctypes::fd_set,
                args[2] as *mut ctypes::fd_set,
                args[3] as *mut ctypes::fd_set,
                args[4] as *mut ctypes::timeval,
                args[5] as *const core::ffi::c_void,
            ) as _,
            #[cfg(feature = "poll")]
            SyscallId::PPOLL => ruxos_posix_api::sys_ppoll(
                args[0] as *mut ctypes::pollfd,
                args[1] as ctypes::nfds_t,
                args[2] as *const ctypes::timespec,
                args[3] as *const ctypes::sigset_t,
                args[4] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::READLINKAT => ruxos_posix_api::sys_readlinkat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as *mut core::ffi::c_char,
                args[3] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::NEWFSTATAT => ruxos_posix_api::sys_newfstatat(
                args[0] as c_int,
                args[1] as *const core::ffi::c_char,
                args[2] as *mut ctypes::kstat,
                args[3] as c_int,
            ) as _,
            #[cfg(feature = "fs")]
            SyscallId::FSTAT => {
                ruxos_posix_api::sys_fstat(args[0] as c_int, args[1] as *mut core::ffi::c_void) as _
            }
            #[cfg(feature = "fs")]
            SyscallId::FSYNC => ruxos_posix_api::sys_fsync(args[0] as c_int) as _,
            SyscallId::GETEUID => ruxos_posix_api::sys_geteuid() as _,
            SyscallId::GETEGID => ruxos_posix_api::sys_getegid() as _,
            #[cfg(feature = "fs")]
            SyscallId::FDATASYNC => ruxos_posix_api::sys_fdatasync(args[0] as c_int) as _,
            #[allow(unreachable_code)]
            #[cfg(not(feature = "multitask"))]
            SyscallId::EXIT => ruxos_posix_api::sys_exit(args[0] as c_int) as _,
            #[allow(unreachable_code)]
            #[cfg(feature = "multitask")]
            SyscallId::EXIT => {
                ruxos_posix_api::sys_pthread_exit(args[0] as *mut core::ffi::c_void) as _
            }
            #[cfg(feature = "multitask")]
            SyscallId::SET_TID_ADDRESS => ruxos_posix_api::sys_set_tid_address(args[0]) as _,
            #[cfg(feature = "multitask")]
            SyscallId::FUTEX => ruxos_posix_api::sys_futex(
                args[0],
                args[1] as _,
                args[2] as _,
                args[3],
                args[4] as _,
                args[5] as _,
            ) as _,
            SyscallId::NANO_SLEEP => ruxos_posix_api::sys_nanosleep(
                args[0] as *const ctypes::timespec,
                args[1] as *mut ctypes::timespec,
            ) as _,
            SyscallId::CLOCK_SETTIME => ruxos_posix_api::sys_clock_settime(
                args[0] as ctypes::clockid_t,
                args[1] as *const ctypes::timespec,
            ) as _,
            SyscallId::CLOCK_GETTIME => ruxos_posix_api::sys_clock_gettime(
                args[0] as ctypes::clockid_t,
                args[1] as *mut ctypes::timespec,
            ) as _,
            SyscallId::SCHED_YIELD => ruxos_posix_api::sys_sched_yield() as _,
            #[cfg(feature = "signal")]
            SyscallId::SIGALTSTACK => ruxos_posix_api::sys_sigaltstack(
                args[0] as *const core::ffi::c_void,
                args[1] as *mut core::ffi::c_void,
            ) as _,
            #[cfg(feature = "signal")]
            SyscallId::RT_SIGACTION => ruxos_posix_api::sys_rt_sigaction(
                args[0] as c_int,
                args[1] as *const ctypes::sigaction,
                args[2] as *mut ctypes::sigaction,
                args[3] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "signal")]
            SyscallId::RT_SIGPROCMASK => ruxos_posix_api::sys_rt_sigprocmask(
                args[0] as c_int,
                args[1] as *const usize,
                args[2] as *mut usize,
                args[3],
            ) as _,
            SyscallId::UNAME => ruxos_posix_api::sys_uname(args[0] as *mut core::ffi::c_void) as _,
            SyscallId::GETRLIMIT => {
                ruxos_posix_api::sys_getrlimit(args[0] as c_int, args[1] as *mut ctypes::rlimit)
                    as _
            }
            SyscallId::SETRLIMIT => {
                ruxos_posix_api::sys_setrlimit(args[0] as c_int, args[1] as *const ctypes::rlimit)
                    as _
            }
            SyscallId::UMASK => ruxos_posix_api::sys_umask(args[0] as ctypes::mode_t) as _,
            #[cfg(feature = "multitask")]
            SyscallId::GETPID => ruxos_posix_api::sys_getpid() as _,
            SyscallId::SYSINFO => {
                ruxos_posix_api::sys_sysinfo(args[0] as *mut ctypes::sysinfo) as _
            }
            #[cfg(feature = "net")]
            SyscallId::SOCKET => {
                ruxos_posix_api::sys_socket(args[0] as c_int, args[1] as c_int, args[2] as c_int)
                    as _
            }
            #[cfg(feature = "net")]
            SyscallId::BIND => ruxos_posix_api::sys_bind(
                args[0] as c_int,
                args[1] as *const ctypes::sockaddr,
                args[2] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::LISTEN => {
                ruxos_posix_api::sys_listen(args[0] as c_int, args[1] as c_int) as _
            }
            #[cfg(feature = "net")]
            SyscallId::ACCEPT => ruxos_posix_api::sys_accept(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::CONNECT => ruxos_posix_api::sys_connect(
                args[0] as c_int,
                args[1] as *const ctypes::sockaddr,
                args[2] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::GETSOCKNAME => ruxos_posix_api::sys_getsockname(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::GETPEERNAME => ruxos_posix_api::sys_getpeername(
                args[0] as c_int,
                args[1] as *mut ctypes::sockaddr,
                args[2] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::SENDTO => ruxos_posix_api::sys_sendto(
                args[0] as c_int,
                args[1] as *const core::ffi::c_void,
                args[2] as ctypes::size_t,
                args[3] as c_int,
                args[4] as *const ctypes::sockaddr,
                args[5] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::RECVFROM => ruxos_posix_api::sys_recvfrom(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2] as ctypes::size_t,
                args[3] as c_int,
                args[4] as *mut ctypes::sockaddr,
                args[5] as *mut ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::SETSOCKOPT => ruxos_posix_api::sys_setsockopt(
                args[0] as c_int,
                args[1] as c_int,
                args[2] as c_int,
                args[3] as *const core::ffi::c_void,
                args[4] as ctypes::socklen_t,
            ) as _,
            #[cfg(feature = "net")]
            SyscallId::SHUTDOWN => {
                ruxos_posix_api::sys_shutdown(args[0] as c_int, args[1] as c_int) as _
            }
            #[cfg(feature = "net")]
            SyscallId::SENDMSG => ruxos_posix_api::sys_sendmsg(
                args[0] as c_int,
                args[1] as *const ctypes::msghdr,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MUNMAP => ruxos_posix_api::sys_munmap(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MREMAP => ruxos_posix_api::sys_mremap(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as ctypes::size_t,
                args[3] as c_int,
                args[4] as *mut core::ffi::c_void,
            ) as _,
            #[cfg(feature = "multitask")]
            SyscallId::CLONE => ruxos_posix_api::sys_clone(
                args[0] as c_int,
                args[1] as *mut core::ffi::c_void,
                args[2] as *mut ctypes::pid_t,
                args[3] as *mut core::ffi::c_void,
                args[4] as *mut ctypes::pid_t,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MMAP => ruxos_posix_api::sys_mmap(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
                args[3] as c_int,
                args[4] as c_int,
                args[5] as ctypes::off_t,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MADVISE => ruxos_posix_api::sys_madvise(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
            ) as _,
            #[cfg(feature = "alloc")]
            SyscallId::MPROTECT => ruxos_posix_api::sys_mprotect(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
            ) as _,
            SyscallId::PRLIMIT64 => ruxos_posix_api::sys_prlimit64(
                args[0] as ctypes::pid_t,
                args[1] as c_int,
                args[2] as *const ctypes::rlimit,
                args[3] as *mut ctypes::rlimit,
            ) as _,
            SyscallId::GETRANDOM => ruxos_posix_api::sys_getrandom(
                args[0] as *mut core::ffi::c_void,
                args[1] as ctypes::size_t,
                args[2] as c_int,
            ) as _,
        }
    }
}
