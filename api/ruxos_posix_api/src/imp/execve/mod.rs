use core::ffi::c_char;

mod auxv;
mod load_elf;
mod stack;

use alloc::vec;

use crate::{
    imp::stat::{sys_getgid, sys_getuid},
    sys_getegid, sys_geteuid,
};

/// int execve(const char *pathname, char *const argv[], char *const envp[] );
pub fn sys_execve(pathname: *const c_char, argv: usize, envp: usize) -> ! {
    use auxv::*;

    let prog = load_elf::ElfProg::new(pathname);

    // get entry
    let mut entry = prog.entry;

    // if interp is needed
    let mut at_base = 0;
    if !prog.interp_path.is_null() {
        let interp_prog = load_elf::ElfProg::new(prog.interp_path);
        entry = interp_prog.entry;
        at_base = interp_prog.base;
        debug!("sys_execve: INTERP base is {:x}", at_base);
    };

    // create stack
    let mut stack = stack::Stack::new();

    let name = prog.name;
    let platform = prog.platform;

    // non 8B info
    stack.push(vec![0u8; 32], 16);
    let p_progname = stack.push(name, 16);
    let _p_plat = stack.push(platform, 16); // platform
    let p_rand = stack.push(prog.rand, 16); // rand

    // auxv
    // TODO: vdso and rand
    // TODO: a way to get pagesz instead of a constant
    let auxv = vec![
        AT_PHDR,
        prog.phdr,
        AT_PHNUM,
        prog.phnum,
        AT_PHENT,
        prog.phent,
        AT_BASE,
        at_base,
        AT_PAGESZ,
        0x1000,
        AT_HWCAP,
        0,
        AT_CLKTCK,
        100,
        AT_FLAGS,
        0,
        AT_ENTRY,
        prog.entry,
        AT_UID,
        sys_getuid() as usize,
        AT_EUID,
        sys_geteuid() as usize,
        AT_EGID,
        sys_getegid() as usize,
        AT_GID,
        sys_getgid() as usize,
        AT_SECURE,
        0,
        AT_EXECFN,
        p_progname,
        AT_RANDOM,
        p_rand,
        AT_SYSINFO_EHDR,
        0,
        AT_IGNORE,
        0,
        AT_NULL,
        0,
    ];

    // handle envs and args
    let mut env_vec = vec![];
    let mut arg_vec = vec![];
    let mut argc = 0;

    let envp = envp as *const usize;
    unsafe {
        let mut i = 0;
        while *envp.add(i) != 0 {
            env_vec.push(*envp.add(i));
            i += 1;
        }
        env_vec.push(0);
    }

    let argv = argv as *const usize;
    unsafe {
        let mut i = 0;
        loop {
            let p = *argv.add(i);
            if p == 0 {
                break;
            }
            arg_vec.push(p);
            argc += 1;
            i += 1;
        }

        arg_vec.push(0);
    }

    // push
    stack.push(auxv, 16);
    stack.push(env_vec, 8);
    stack.push(arg_vec, 8);
    let _sp = stack.push(vec![argc as usize], 8);

    // try run
    debug!(
        "sys_execve: run at entry 0x{entry:x}, then it will jump to 0x{:x} ",
        prog.entry
    );

    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("
         mov sp, {}
         blr {}
     ",
        in(reg)_sp,
        in(reg)entry,
        );
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("
         mov rsp, {}
         jmp {}
     ",
        in(reg)_sp,
        in(reg)entry,
        );
    }

    unreachable!("sys_execve: unknown arch");
}
