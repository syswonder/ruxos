/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::sync::Arc;
use axfs_vfs::{RelPath, VfsNodeType, VfsOps, VfsResult};

#[cfg(feature = "alloc")]
use crate::arch::{get_cpuinfo, get_meminfo};
use crate::fs;

#[cfg(feature = "devfs")]
pub(crate) fn devfs() -> Arc<fs::devfs::DeviceFileSystem> {
    let null = fs::devfs::NullDev;
    let zero = fs::devfs::ZeroDev;
    let random = fs::devfs::RandomDev;
    let urandom = fs::devfs::RandomDev;
    let pts = fs::devfs::init_pts();
    let devfs = fs::devfs::DeviceFileSystem::new();
    devfs.add("null", Arc::new(null));
    devfs.add("zero", Arc::new(zero));
    devfs.add("random", Arc::new(random));
    devfs.add("urandom", Arc::new(urandom));
    devfs.add("pts", pts);
    Arc::new(devfs)
}

#[cfg(feature = "ramfs")]
pub(crate) fn ramfs() -> Arc<fs::ramfs::RamFileSystem> {
    Arc::new(fs::ramfs::RamFileSystem::new())
}

#[cfg(feature = "procfs")]
pub(crate) fn procfs() -> VfsResult<Arc<fs::ramfs::RamFileSystem>> {
    let procfs = fs::ramfs::RamFileSystem::new();
    let proc_root = procfs.root_dir();

    #[cfg(feature = "alloc")]
    {
        // Create /proc/cpuinfo
        proc_root.create(&RelPath::new("cpuinfo"), VfsNodeType::File)?;
        let file_cpuinfo = proc_root.clone().lookup(&RelPath::new("cpuinfo"))?;
        file_cpuinfo.write_at(0, get_cpuinfo().as_bytes())?;

        // Create /proc/meminfo
        proc_root.create(&RelPath::new("meminfo"), VfsNodeType::File)?;
        let file_meminfo = proc_root.clone().lookup(&RelPath::new("meminfo"))?;
        file_meminfo.write_at(0, get_meminfo().as_bytes())?;
    }

    // Create /proc/sys/net/core/somaxconn
    proc_root.create_recursive(&RelPath::new("sys/net/core/somaxconn"), VfsNodeType::File)?;
    let file_somaxconn = proc_root
        .clone()
        .lookup(&RelPath::new("sys/net/core/somaxconn"))?;
    file_somaxconn.write_at(0, b"4096\n")?;

    // Create /proc/sys/vm/overcommit_memory
    proc_root.create_recursive(&RelPath::new("sys/vm/overcommit_memory"), VfsNodeType::File)?;
    let file_over = proc_root
        .clone()
        .lookup(&RelPath::new("sys/vm/overcommit_memory"))?;
    file_over.write_at(0, b"0\n")?;

    // Create /proc/self/stat
    proc_root.create_recursive(&RelPath::new("self/stat"), VfsNodeType::File)?;

    Ok(Arc::new(procfs))
}

#[cfg(feature = "sysfs")]
pub(crate) fn sysfs() -> VfsResult<Arc<fs::ramfs::RamFileSystem>> {
    let sysfs = fs::ramfs::RamFileSystem::new();
    let sys_root = sysfs.root_dir();

    debug!("sysfs: {:?}", sys_root.get_attr());

    // Create /sys/kernel/mm/transparent_hugepage/enabled
    sys_root.create_recursive(
        &RelPath::new("kernel/mm/transparent_hugepage/enabled"),
        VfsNodeType::File,
    )?;
    let file_hp = sys_root
        .clone()
        .lookup(&RelPath::new("kernel/mm/transparent_hugepage/enabled"))?;
    file_hp.write_at(0, b"always [madvise] never\n")?;

    // Create /sys/devices/system/clocksource/clocksource0/current_clocksource
    sys_root.create_recursive(
        &RelPath::new("devices/system/clocksource/clocksource0/current_clocksource"),
        VfsNodeType::File,
    )?;
    let file_cc = sys_root.clone().lookup(&RelPath::new(
        "devices/system/clocksource/clocksource0/current_clocksource",
    ))?;
    file_cc.write_at(0, b"tsc\n")?;

    Ok(Arc::new(sysfs))
}

#[cfg(feature = "etcfs")]
pub(crate) fn etcfs() -> VfsResult<Arc<fs::ramfs::RamFileSystem>> {
    let etcfs = fs::ramfs::RamFileSystem::new();
    let etc_root = etcfs.root_dir();

    // Create /etc/passwd, and /etc/hosts
    etc_root.create(&RelPath::new("passwd"), VfsNodeType::File)?;
    let file_passwd = etc_root.clone().lookup(&RelPath::new("passwd"))?;
    // format: username:password:uid:gid:allname:homedir:shell
    file_passwd.write_at(
        0,
        b"root:x:0:0:root:/root:/bin/busybox\n\
        syswonder:x:1000:1000:root:/root:/bin/busybox\n",
    )?;

    // Create /etc/group
    etc_root.create(&RelPath::new("group"), VfsNodeType::File)?;
    let file_group = etc_root.clone().lookup(&RelPath::new("group"))?;
    file_group.write_at(0, b"root:x:0:\n")?;

    // Create /etc/localtime
    etc_root.create(&RelPath::new("localtime"), VfsNodeType::File)?;

    // Create /etc/hosts
    etc_root.create(&RelPath::new("hosts"), VfsNodeType::File)?;
    let file_hosts = etc_root.clone().lookup(&RelPath::new("hosts"))?;
    file_hosts.write_at(
        0,
        b"127.0.0.1	localhost\n\n\
        ::1 ip6-localhost ip6-loopback \n\
        fe00::0 ip6-localnet \n\
        ff00::0 ip6-mcastprefix \n\
        ff02::1 ip6-allnodes \n\
        ff02::2 ip6-allrouters \n\
        ff02::3 ip6-allhosts\n",
    )?;

    etc_root.create(&RelPath::new("services"), VfsNodeType::File)?;
    let file_services = etc_root.clone().lookup(&RelPath::new("services"))?;
    file_services.write_at(0, b"ssh		22/tcp")?;

    // Create /etc/resolv.conf
    etc_root.create(&RelPath::new("resolv.conf"), VfsNodeType::File)?;
    let file_resolv = etc_root.clone().lookup(&RelPath::new("resolv.conf"))?;
    file_resolv.write_at(
        0,
        b"nameserver 8.8.8.8\n\
        nameserver 114.114.114.114\n\
        options edns0 trust-ad\n\
        search lan\n
        ",
    )?;

    Ok(Arc::new(etcfs))
}
