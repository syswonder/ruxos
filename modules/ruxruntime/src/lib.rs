/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Runtime library of [Ruxos](https://github.com/syswonder/ruxos).
//!
//! Any application uses Ruxos should link this library. It does some
//! initialization work before entering the application's `main` function.
//!
//! # Cargo Features
//!
//! - `alloc`: Enable global memory allocator.
//! - `paging`: Enable page table manipulation support.
//! - `tls`: Enable thread local storage support.
//! - `irq`: Enable interrupt handling support.
//! - `multitask`: Enable multi-threading support.
//! - `smp`: Enable SMP (symmetric multiprocessing) support.
//! - `fs`: Enable filesystem support.
//! - `blkfs`: Enable disk filesystem.
//! - `signal`: Enable signal support
//! - `net`: Enable networking support.
//! - `display`: Enable graphics support.
//! - `virtio-9p`: Enable virtio-based 9pfs support.
//! - `net-9p`: Enable net-based 9pfs support.
//! - `musl`: Enable musl libc support.
//!
//! All the features are optional and disabled by default.

#![cfg_attr(not(test), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate axlog;

#[cfg(all(target_os = "none", not(test)))]
mod lang_items;
#[cfg(feature = "signal")]
mod signal;

#[cfg(not(feature = "musl"))]
mod trap;

#[cfg(feature = "smp")]
mod mp;

#[cfg(feature = "smp")]
pub use self::mp::rust_main_secondary;

#[cfg(feature = "signal")]
pub use self::signal::{rx_sigaction, Signal};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod env;
#[cfg(feature = "alloc")]
pub use self::env::{argv, environ, environ_iter, RUX_ENVIRON};
#[cfg(feature = "alloc")]
use self::env::{boot_add_environ, init_argv};
use core::ffi::{c_char, c_int};

const LOGO: &str = r#"
8888888b.                     .d88888b.   .d8888b.  
888   Y88b                   d88P" "Y88b d88P  Y88b 
888    888                   888     888 Y88b.      
888   d88P 888  888 888  888 888     888  "Y888b.   
8888888P"  888  888 `Y8bd8P' 888     888     "Y88b. 
888 T88b   888  888   X88K   888     888       "888 
888  T88b  Y88b 888 .d8""8b. Y88b. .d88P Y88b  d88P 
888   T88b  "Y88888 888  888  "Y88888P"   "Y8888P" 
"#;

#[no_mangle]
extern "C" fn init_dummy() {}

#[no_mangle]
extern "C" fn fini_dummy() {}

#[no_mangle]
extern "C" fn ldso_dummy() {}

extern "C" {
    fn main(argc: c_int, argv: *mut *mut c_char) -> c_int;
    fn __libc_start_main(
        main: unsafe extern "C" fn(argc: c_int, argv: *mut *mut c_char) -> c_int,
        argc: c_int,
        argv: *mut *mut c_char,
        init_dummy: extern "C" fn(),
        fini_dummy: extern "C" fn(),
        ldso_dummy: extern "C" fn(),
    ) -> c_int;
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn console_write_str(s: &str) {
        ruxhal::console::write_bytes(s.as_bytes());
    }

    fn current_time() -> core::time::Duration {
        ruxhal::time::current_time()
    }

    fn current_cpu_id() -> Option<usize> {
        #[cfg(feature = "smp")]
        if is_init_ok() {
            Some(ruxhal::cpu::this_cpu_id())
        } else {
            None
        }
        #[cfg(not(feature = "smp"))]
        Some(0)
    }

    fn current_task_id() -> Option<u64> {
        if is_init_ok() {
            #[cfg(feature = "multitask")]
            {
                ruxtask::current_may_uninit().map(|curr| curr.id().as_u64())
            }
            #[cfg(not(feature = "multitask"))]
            None
        } else {
            None
        }
    }
}

use core::sync::atomic::{AtomicUsize, Ordering};

static INITED_CPUS: AtomicUsize = AtomicUsize::new(0);

fn is_init_ok() -> bool {
    INITED_CPUS.load(Ordering::Acquire) == ruxconfig::SMP
}

/// The main entry point of the Ruxos runtime.
///
/// It is called from the bootstrapping code in [ruxhal]. `cpu_id` is the ID of
/// the current CPU, and `dtb` is the address of the device tree blob. It
/// finally calls the application's `main` function after all initialization
/// work is done.
///
/// In multi-core environment, this function is called on the primary CPU,
/// and the secondary CPUs call [`rust_main_secondary`].
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn rust_main(cpu_id: usize, dtb: usize) -> ! {
    ax_println!("{}", LOGO);
    ax_println!(
        "\
        arch = {}\n\
        platform = {}\n\
        target = {}\n\
        smp = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("RUX_ARCH").unwrap_or(""),
        option_env!("RUX_PLATFORM").unwrap_or(""),
        option_env!("RUX_TARGET").unwrap_or(""),
        option_env!("RUX_SMP").unwrap_or(""),
        option_env!("RUX_MODE").unwrap_or(""),
        option_env!("RUX_LOG").unwrap_or(""),
    );

    axlog::init();
    axlog::set_max_level(option_env!("RUX_LOG").unwrap_or("")); // no effect if set `log-level-*` features
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", cpu_id, dtb);

    info!("Found physcial memory regions:");
    for r in ruxhal::mem::memory_regions() {
        info!(
            "  [{:x?}, {:x?}) {} ({:?})",
            r.paddr,
            r.paddr + r.size,
            r.name,
            r.flags
        );
    }

    #[cfg(feature = "alloc")]
    init_allocator();

    #[cfg(feature = "paging")]
    {
        info!("Initialize kernel page table...");
        remap_kernel_memory().expect("remap kernel memoy failed");
    }

    info!("Initialize platform devices...");
    ruxhal::platform_init();

    #[cfg(feature = "multitask")]
    {
        ruxtask::init_scheduler();
        #[cfg(feature = "musl")]
        ruxfutex::init_futex();
    }

    #[cfg(any(feature = "fs", feature = "net", feature = "display"))]
    {
        #[allow(unused_variables)]
        let all_devices = ruxdriver::init_drivers();

        #[cfg(feature = "net")]
        axnet::init_network(all_devices.net);

        #[cfg(feature = "fs")]
        {
            extern crate alloc;
            use alloc::vec::Vec;
            // By default, mount_points[0] will be rootfs
            let mut mount_points: Vec<ruxfs::MountPoint> = Vec::new();

            //setup ramfs as rootfs if no other filesystem can be mounted
            #[cfg(not(any(feature = "blkfs", feature = "virtio-9p", feature = "net-9p")))]
            mount_points.push(ruxfs::init_tempfs());

            // setup and initialize blkfs as one mountpoint for rootfs
            #[cfg(feature = "blkfs")]
            mount_points.push(ruxfs::init_blkfs(all_devices.block));

            // setup and initialize 9pfs as mountpoint
            #[cfg(feature = "virtio-9p")]
            mount_points.push(rux9p::init_virtio_9pfs(
                all_devices._9p,
                option_env!("RUX_ANAME_9P").unwrap_or(""),
                option_env!("RUX_PROTOCOL_9P").unwrap_or(""),
            ));
            #[cfg(feature = "net-9p")]
            mount_points.push(rux9p::init_net_9pfs(
                option_env!("RUX_9P_ADDR").unwrap_or(""),
                option_env!("RUX_ANAME_9P").unwrap_or(""),
                option_env!("RUX_PROTOCOL_9P").unwrap_or(""),
            ));
            ruxfs::prepare_commonfs(&mut mount_points);

            // setup and initialize rootfs
            ruxfs::init_filesystems(mount_points);
        }

        #[cfg(feature = "display")]
        ruxdisplay::init_display(all_devices.display);
    }

    #[cfg(feature = "smp")]
    self::mp::start_secondary_cpus(cpu_id);

    #[cfg(feature = "irq")]
    {
        info!("Initialize interrupt handlers...");
        init_interrupt();
    }

    #[cfg(all(feature = "tls", not(feature = "multitask")))]
    {
        info!("Initialize thread local storage...");
        init_tls();
    }

    info!("Primary CPU {} init OK.", cpu_id);
    INITED_CPUS.fetch_add(1, Ordering::Relaxed);

    while !is_init_ok() {
        core::hint::spin_loop();
    }

    // environ variables and Command line parameters initialization
    #[cfg(feature = "alloc")]
    unsafe {
        let mut argc: c_int = 0;
        init_cmdline(&mut argc);
        #[cfg(not(feature = "musl"))]
        main(argc, argv);
        #[cfg(feature = "musl")]
        __libc_start_main(main, argc, argv, init_dummy, fini_dummy, ldso_dummy);
    }

    #[cfg(not(feature = "alloc"))]
    unsafe {
        #[cfg(not(feature = "musl"))]
        main(0, core::ptr::null_mut());

        #[cfg(feature = "musl")]
        __libc_start_main(
            main,
            0,
            core::ptr::null_mut(),
            init_dummy,
            fini_dummy,
            ldso_dummy,
        )
    };

    #[cfg(feature = "multitask")]
    ruxtask::exit(0);
    #[cfg(not(feature = "multitask"))]
    {
        debug!("main task exited: exit_code={}", 0);
        ruxhal::misc::terminate();
    }
}

#[cfg(feature = "alloc")]
cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        fn get_boot_str() -> &'static str {
            let cmdline_buf: &[u8] = unsafe { &ruxhal::COMLINE_BUF };
            let mut len = 0;
            for c in cmdline_buf.iter() {
                if *c == 0 {
                    break;
                }
                len += 1;
            }
            core::str::from_utf8(&cmdline_buf[..len]).unwrap()
        }
    } else {
        fn get_boot_str() -> &'static str {
            dtb::get_node("chosen").unwrap().prop("bootargs").str()
        }
    }
}

// initialize environ variables and Command line parameters
#[cfg(feature = "alloc")]
fn init_cmdline(argc: &mut c_int) {
    use alloc::vec::Vec;
    let mut boot_str = get_boot_str();
    (_, boot_str) = match boot_str.split_once(';') {
        Some((a, b)) => (a, b),
        None => ("", ""),
    };
    let (args, envs) = match boot_str.split_once(';') {
        Some((a, e)) => (a, e),
        None => ("", ""),
    };
    // set env
    let envs: Vec<&str> = envs.split(',').collect();
    for i in envs {
        boot_add_environ(i);
    }
    // set args
    unsafe {
        RUX_ENVIRON.push(core::ptr::null_mut());
        environ = RUX_ENVIRON.as_mut_ptr();
        let args: Vec<&str> = args.split(',').filter(|i| !i.is_empty()).collect();
        *argc = args.len() as c_int;
        init_argv(args);
    }
}

#[cfg(feature = "alloc")]
fn init_allocator() {
    use ruxhal::mem::{memory_regions, phys_to_virt, MemRegionFlags};

    info!("Initialize global memory allocator...");
    info!("  use {} allocator.", axalloc::global_allocator().name());

    let mut max_region_size = 0;
    let mut max_region_paddr = 0.into();
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.size > max_region_size {
            max_region_size = r.size;
            max_region_paddr = r.paddr;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
            axalloc::global_init(phys_to_virt(r.paddr).as_usize(), r.size);
            break;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr != max_region_paddr {
            axalloc::global_add_memory(phys_to_virt(r.paddr).as_usize(), r.size)
                .expect("add heap memory region failed");
        }
    }
}

#[cfg(feature = "paging")]
fn remap_kernel_memory() -> Result<(), ruxhal::paging::PagingError> {
    use lazy_init::LazyInit;
    use ruxhal::mem::{memory_regions, phys_to_virt};
    use ruxhal::paging::PageTable;

    static KERNEL_PAGE_TABLE: LazyInit<PageTable> = LazyInit::new();

    if ruxhal::cpu::this_cpu_is_bsp() {
        let mut kernel_page_table = PageTable::try_new()?;
        for r in memory_regions() {
            kernel_page_table.map_region(
                phys_to_virt(r.paddr),
                r.paddr,
                r.size,
                r.flags.into(),
                true,
            )?;
        }
        KERNEL_PAGE_TABLE.init_by(kernel_page_table);
    }

    unsafe { ruxhal::arch::write_page_table_root(KERNEL_PAGE_TABLE.root_paddr()) };
    Ok(())
}

#[cfg(feature = "irq")]
fn init_interrupt() {
    use ruxhal::time::TIMER_IRQ_NUM;

    // Setup timer interrupt handler
    const PERIODIC_INTERVAL_NANOS: u64 =
        ruxhal::time::NANOS_PER_SEC / ruxconfig::TICKS_PER_SEC as u64;

    #[percpu::def_percpu]
    static NEXT_DEADLINE: u64 = 0;

    fn update_timer() {
        let now_ns = ruxhal::time::current_time_nanos();
        // Safety: we have disabled preemption in IRQ handler.
        let mut deadline = unsafe { NEXT_DEADLINE.read_current_raw() };
        if now_ns >= deadline {
            deadline = now_ns + PERIODIC_INTERVAL_NANOS;
        }
        unsafe { NEXT_DEADLINE.write_current_raw(deadline + PERIODIC_INTERVAL_NANOS) };
        ruxhal::time::set_oneshot_timer(deadline);
    }

    #[cfg(feature = "signal")]
    fn do_signal() {
        let now_ns = ruxhal::time::current_time_nanos();
        // timer signal num
        let timers = [14, 26, 27];
        for (which, timer) in timers.iter().enumerate() {
            let mut ddl = Signal::timer_deadline(which, None).unwrap();
            let interval = Signal::timer_interval(which, None).unwrap();
            if ddl != 0 && now_ns >= ddl {
                Signal::signal(*timer, true);
                if interval == 0 {
                    ddl = 0;
                } else {
                    ddl += interval;
                }
                Signal::timer_deadline(which, Some(ddl));
            }
        }
        let signal = Signal::signal(-1, true).unwrap();
        for signum in 0..32 {
            if signal & (1 << signum) != 0
            /* TODO: && support mask */
            {
                Signal::sigaction(signum as u8, None, None);
                Signal::signal(signum as i8, false);
            }
        }
    }

    ruxhal::irq::register_handler(TIMER_IRQ_NUM, || {
        update_timer();
        #[cfg(feature = "signal")]
        if ruxhal::cpu::this_cpu_is_bsp() {
            do_signal();
        }
        #[cfg(feature = "multitask")]
        ruxtask::on_timer_tick();
    });

    // Enable IRQs before starting app
    ruxhal::arch::enable_irqs();
}

#[cfg(all(feature = "tls", not(feature = "multitask")))]
fn init_tls() {
    let main_tls = ruxhal::tls::TlsArea::alloc();
    unsafe { ruxhal::arch::write_thread_pointer(main_tls.tls_ptr() as usize) };
    core::mem::forget(main_tls);
}
