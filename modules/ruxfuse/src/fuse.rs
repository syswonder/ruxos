/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! FUSE filesystem used by [RuxOS](https://github.com/syswonder/ruxos).
//!
//! The implementation is based on [`axfs_vfs`].

use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use log::*;
use ruxtask::{current, WaitQueue};
use spinlock::SpinNoIrq;

use axfs_vfs::{RelPath, VfsDirEntry, VfsError, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps};
use ruxfs::devfuse::{FUSEFLAG, FUSE_VEC};
use ruxfs::fuse_st::{
    FuseAccessIn, FuseAttr, FuseAttrOut, FuseCreateIn, FuseDirent, FuseEntryOut, FuseFlushIn,
    FuseForgetIn, FuseGetattrIn, FuseInHeader, FuseInitIn, FuseInitOut, FuseLseekIn, FuseLseekOut,
    FuseMkdirIn, FuseMknodIn, FuseOpcode, FuseOpenIn, FuseOpenOut, FuseOutHeader, FuseReadIn,
    FuseReleaseIn, FuseRename2In, FuseRenameIn, FuseStatfsOut, FuseWriteIn, FuseWriteOut,
};
use spin::{once::Once, RwLock};

/// Unique id for FUSE operations.
pub static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);
/// A flag for FuseRename operation.
pub static NEWID: AtomicI32 = AtomicI32::new(-1);
/// A flag to indicate whether FUSE is initialized.
pub static INITFLAG: AtomicI32 = AtomicI32::new(1);
/// A static wait queue for FUSE operations.
pub static WQ: WaitQueue = WaitQueue::new();

/// It implements [`axfs_vfs::VfsOps`].
pub struct FuseFS {
    parent: Once<VfsNodeRef>,
    root: Arc<FuseNode>,
}

impl Default for FuseFS {
    fn default() -> Self {
        Self::new()
    }
}

impl FuseFS {
    /// Create a new instance.
    pub fn new() -> Self {
        debug!("fusefs new...");
        Self {
            parent: Once::new(),
            root: FuseNode::new(None, 1, FuseAttr::default(), 0, 0),
        }
    }
}

impl VfsOps for FuseFS {
    fn mount(&self, parent: VfsNodeRef) -> VfsResult {
        self.root.set_parent(Some(self.parent.call_once(|| parent)));
        Ok(())
    }

    fn umount(&self) -> VfsResult {
        debug!("fusefs umount...");
        self.root.destroy()
    }

    fn root_dir(&self) -> VfsNodeRef {
        debug!("fusefs root_dir...");
        self.root.clone()
    }
}

/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct FuseNode {
    this: Weak<FuseNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    inode: SpinNoIrq<u64>,
    attr: SpinNoIrq<FuseAttr>,
    nlink: SpinNoIrq<u32>,
    flags: SpinNoIrq<u32>,
    fh: SpinNoIrq<u64>,
}

impl FuseNode {
    pub(super) fn new(
        parent: Option<Weak<dyn VfsNodeOps>>,
        inode: u64,
        attr: FuseAttr,
        nlink: u32,
        fh: u64,
    ) -> Arc<Self> {
        debug!("fuse_node new inode: {inode:?}, nlink: {nlink:?}");
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            inode: SpinNoIrq::new(inode),
            attr: SpinNoIrq::new(attr),
            nlink: SpinNoIrq::new(nlink),
            flags: SpinNoIrq::new(0x8000),
            fh: SpinNoIrq::new(fh),
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        debug!("fuse_node set_parent...");
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Get inode of this FuseNode.
    pub fn get_node_inode(&self) -> u64 {
        let inode_guard = self.inode.lock();
        *inode_guard
    }

    /// Get attr of this FuseNode.
    pub fn get_node_attr(&self) -> FuseAttr {
        let attr_guard = self.attr.lock();
        *attr_guard
    }

    /// Get nlink of this FuseNode.
    pub fn get_node_nlink(&self) -> u32 {
        let nlink_guard = self.nlink.lock();
        *nlink_guard
    }

    /// Get flags of this FuseNode.
    pub fn get_node_flags(&self) -> u32 {
        let flags_guard = self.flags.lock();
        *flags_guard
    }

    /// Get file handle (fh) of this FuseNode.
    pub fn get_fh(&self) -> u64 {
        let fh_guard = self.fh.lock();
        *fh_guard
    }

    /// Set inode of this FuseNode.
    pub fn set_node_inode(&self, inode: u64) {
        let mut inode_guard = self.inode.lock();
        *inode_guard = inode;
    }

    /// Set attr of this FuseNode.
    pub fn set_node_attr(&self, attr: FuseAttr) {
        let mut attr_guard = self.attr.lock();
        *attr_guard = attr;
    }

    /// Set nlink of this FuseNode.
    pub fn set_node_nlink(&self, nlink: u32) {
        let mut nlink_guard = self.nlink.lock();
        *nlink_guard = nlink;
    }

    /// Set flags of this FuseNode.
    pub fn set_node_flags(&self, flags: u32) {
        let mut flags_guard = self.flags.lock();
        *flags_guard = flags;
    }

    /// Set file handle (fh) of this FuseNode.
    pub fn set_fh(&self, fh: u64) {
        let mut fh_guard = self.fh.lock();
        *fh_guard = fh;
    }

    /// Get inode of this FuseNode.
    pub fn find_inode(&self, path: &str) -> Option<u64> {
        let (mut name, mut raw_rest) = split_path(path);
        if raw_rest.is_none() {
            return self.get_inode();
        }
        let mut node = self.try_get(&RelPath::new(".")).unwrap();
        while raw_rest.is_some() {
            let rest = raw_rest.unwrap();
            if !name.is_empty() && name != "." {
                node = node.lookup(&RelPath::new(name)).unwrap();
            }
            (name, raw_rest) = split_path(rest);
        }

        node.get_inode()
    }

    /// Get final name of the path.
    #[allow(clippy::only_used_in_recursion)]
    pub fn get_final_name(&self, path: &str) -> Option<String> {
        let (name, rest) = split_path(path);
        if rest.is_none() {
            return Some(name.to_string());
        }
        self.get_final_name(rest.unwrap())
    }

    /// Check if the node is a directory.
    pub fn is_dir(&self) -> bool {
        let attr_guard = self.attr.lock();
        let attr = &*attr_guard;
        let mode = attr.get_mode();
        // 0x8124 => file ( can read => 0x8000
        // 0x81a4 => file ( can read and write => 0x8001
        // 0x41ed => directory ( can read and execute
        // S_IFDIR = 0x4000
        // S_IFREG = 0x8000
        mode & 0x4000 == 0x4000
    }

    /// Get file flags for this FuseNode.
    pub fn file_flags(&self) -> u32 {
        let attr_guard = self.attr.lock();
        let attr = &*attr_guard;
        let mode = attr.get_mode();
        match mode & 0x1c0 {
            0x80 => 0x8001,
            0x100 => 0x8000, // 0x8124
            0x180 => 0x8001, // 0x81a4
            0x1c0 => 0x8002, // 0x81ed?
            _ => 0x8000,
        }
    }

    /// Get directory flags for this FuseNode.
    pub fn dir_flags(&self) -> u32 {
        let attr_guard = self.attr.lock();
        let attr = &*attr_guard;
        let mode = attr.get_mode();
        match mode & 0x1c0 {
            0x20 => 0x18801, // O_WRONLY
            0x40 => 0x18800, // O_RDONLY
            0x80 => 0x18802, // O_RDWR
            _ => 0x18800,
        }
    }

    /// Check if already initialized.
    pub fn check_init(&self) {
        let f1 = INITFLAG.load(Ordering::SeqCst);
        if f1 == 1 {
            INITFLAG.store(0, Ordering::Relaxed);
            UNIQUE_ID.store(0, Ordering::Relaxed);
            self.init();
        }
    }

    /// FuseInit = 26
    pub fn init(&self) {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node INIT({:?}) here...",
            FuseOpcode::FuseInit as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        debug!("pid = {pid:?}, inode = {nodeid:?}");
        let fusein = FuseInHeader::new(
            104,
            FuseOpcode::FuseInit as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 104];
        fusein.write_to(&mut fusebuf);
        let initin = FuseInitIn::new(7, 38, 0x00020000, 0x33fffffb, 0, [0; 11]);
        initin.write_to(&mut fusebuf[40..]);
        fusein.print();
        initin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at init in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseInit as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at init is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 80];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to init: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();
        // init_flag: 0x40F039
        let initout = FuseInitOut::read_from(&outbuf[16..]);
        initout.print();

        if initout.get_major() != 7 || initout.get_minor() != 38 {
            warn!(
                "fuse_node init unsupport version, major = {:?}, minor = {:?}",
                initout.get_major(),
                initout.get_minor()
            );
        } else if initout.get_flags() != 0x40f039 {
            warn!(
                "fuse_node init unsupport flags = {:#x}",
                initout.get_flags()
            );
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node init finish successfully...");
    }

    /// FuseLookup = 1
    fn try_get(&self, path: &RelPath) -> VfsResult<VfsNodeRef> {
        self.check_init();

        let (name, raw_rest) = split_path(path);
        if let Some(rest) = raw_rest {
            if name.is_empty() || name == "." {
                return self.try_get(&RelPath::new(rest));
            }
            let node = self.try_get(&RelPath::new(name))?;
            return node.lookup(&RelPath::new(rest));
        }

        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node LOOKUP({:?}) {:?} here...",
            FuseOpcode::FuseLookup as u32,
            path
        );

        let lookup_error;
        let mut entryout = FuseEntryOut::default();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let path_len = path.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            41 + path_len as u32,
            FuseOpcode::FuseLookup as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf[0..40]);
        fusebuf[40..40 + path_len].copy_from_slice(path.as_bytes());
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at lookup in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseLookup as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at lookup is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 144];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to lookup: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf[..16]);
        fuseout.print();

        if fuseout.is_ok() {
            entryout = FuseEntryOut::read_from(&outbuf[16..]);
            entryout.print();
            lookup_error = 1;
        } else {
            lookup_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node lookup finish successfully...");

        if lookup_error < 0 {
            match lookup_error {
                -2 => return Err(VfsError::NotFound),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        match name {
            "" | "." => {
                self.set_node_inode(entryout.get_nodeid());
                self.set_node_attr(entryout.get_attr());
                self.set_node_nlink(entryout.get_nlink());
                let parent = match self.parent() {
                    Some(_) => Some(Arc::downgrade(&self.parent().unwrap())),
                    None => None,
                };
                let node = FuseNode::new(
                    parent,
                    entryout.get_nodeid(),
                    entryout.get_attr(),
                    entryout.get_nlink(),
                    0,
                );
                Ok(node)
            }
            ".." => {
                let node = FuseNode::new(
                    None,
                    entryout.get_nodeid(),
                    entryout.get_attr(),
                    entryout.get_nlink(),
                    0,
                );
                Ok(node)
            }
            _ => {
                let node = FuseNode::new(
                    Some(self.this.clone()),
                    entryout.get_nodeid(),
                    entryout.get_attr(),
                    entryout.get_nlink(),
                    0,
                );
                Ok(node)
            }
        }
    }

    /// FuseOpendir = 27
    pub fn open_dir(&self) -> Result<Option<Arc<dyn VfsNodeOps>>, VfsError> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node OPENDIR({:?}) here...",
            FuseOpcode::FuseOpendir as u32
        );

        let opendir_error;
        let mut opendirout = FuseOpenOut::default();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            48,
            FuseOpcode::FuseOpendir as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 48];
        fusein.write_to(&mut fusebuf);
        let openin = FuseOpenIn::new(0x18800, 0);
        openin.write_to(&mut fusebuf[40..]);
        fusein.print();
        openin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at open_dir in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseOpendir as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at open_dir is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 32];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to open_dir: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            opendirout = FuseOpenOut::read_from(&outbuf[16..]);
            opendirout.print();
            opendir_error = 1;
        } else {
            opendir_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node open_dir finish successfully...");

        if opendir_error < 0 {
            match opendir_error {
                -13 => return Err(VfsError::PermissionDenied),
                -20 => return Err(VfsError::NotADirectory),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        let mut fh_guard = self.fh.lock();
        let fh = &mut *fh_guard;
        *fh = opendirout.get_fh();

        debug!("fh = {fh:#x}");
        debug!("opendirout.fh = {:#x}", opendirout.get_fh());

        Ok(None)
    }

    /// FuseReleasedir = 28
    pub fn release_dir(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node RELEASEDIR({:?}) here...",
            FuseOpcode::FuseReleasedir as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            64,
            FuseOpcode::FuseReleasedir as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 64];
        fusein.write_to(&mut fusebuf);
        let releasein = FuseReleaseIn::new(fh, 0x18800, 0, 0);
        releasein.write_to(&mut fusebuf[40..]);
        fusein.print();
        releasein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at release_dir in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseReleasedir as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at release_dir is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to release_dir: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let releasedir_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node release_dir finish successfully...");

        if releasedir_error < 0 {
            match releasedir_error {
                -13 => Err(VfsError::PermissionDenied),
                -20 => Err(VfsError::NotADirectory),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(())
        }
    }

    /// FuseForget = 2
    pub fn forget(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node FORGET({:?}) here...",
            FuseOpcode::FuseForget as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            48,
            FuseOpcode::FuseForget as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 48];
        fusein.write_to(&mut fusebuf);
        let forgetin = FuseForgetIn::new(4);
        forgetin.write_to(&mut fusebuf[40..]);
        fusein.print();
        forgetin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at forget in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseForget as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at forget is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to forget: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let forget_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node forget finish successfully...");

        if forget_error < 0 {
            match forget_error {
                -13 => Err(VfsError::PermissionDenied),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(())
        }
    }

    /// FuseSetattr = 4
    pub fn set_attr(&self, attr: &FuseAttr, to_set: u32) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node SETATTR({:?}) here...",
            FuseOpcode::FuseSetattr as u32
        );

        let setattr_error;
        let mut attrout = FuseAttrOut::default();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            128,
            FuseOpcode::FuseSetattr as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 128];
        fusein.write_to(&mut fusebuf);
        let mut attrin = FuseAttr::default();
        if to_set & 0x1 != 0 {
            attrin.set_mode(attr.get_mode());
        }
        if to_set & 0x2 != 0 {
            attrin.set_uid(attr.get_uid());
        }
        if to_set & 0x4 != 0 {
            attrin.set_gid(attr.get_gid());
        }
        if to_set & 0x8 != 0 {
            attrin.set_size(attr.get_size());
        }
        if to_set & 0x10 != 0 {
            attrin.set_atime(attr.get_atime());
        }
        if to_set & 0x20 != 0 {
            attrin.set_mtime(attr.get_mtime());
        }
        if to_set & 0x40 != 0 {
            attrin.set_ctime(attr.get_ctime());
        }
        if to_set & 0x80 != 0 {
            attrin.set_atimensec(attr.get_atimensec());
        }
        if to_set & 0x100 != 0 {
            attrin.set_mtimensec(attr.get_mtimensec());
        }
        if to_set & 0x200 != 0 {
            attrin.set_ctimensec(attr.get_ctimensec());
        }
        attrin.write_to(&mut fusebuf[40..]);
        fusein.print();
        attrin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at setattr in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseSetattr as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at setattr is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 120];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to setattr: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            attrout = FuseAttrOut::read_from(&outbuf[16..]);
            attrout.print();
            setattr_error = 1;
        } else {
            setattr_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node setattr finish successfully...");

        if setattr_error < 0 {
            match setattr_error {
                -13 => return Err(VfsError::PermissionDenied),
                -22 => return Err(VfsError::InvalidInput),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        if attrout.get_attr_valid() != 0 {
            let mut attr_guard = self.attr.lock();
            *attr_guard = attrout.get_attr();
        }

        Ok(())
    }

    /// FuseReadlink = 5
    pub fn readlink(&self) -> VfsResult<String> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node READLINK({:?}) here...",
            FuseOpcode::FuseReadlink as u32
        );

        let readlink_error;
        let mut readlinkout = String::new();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            40,
            FuseOpcode::FuseReadlink as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 40];
        fusein.write_to(&mut fusebuf);
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at readlink in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseReadlink as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at readlink is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 144];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to readlink: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            readlinkout = String::from_utf8_lossy(&outbuf[16..]).to_string();
            readlink_error = 1;
        } else {
            readlink_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node readlink finish successfully...");

        if readlink_error < 0 {
            match readlink_error {
                -13 => return Err(VfsError::PermissionDenied),
                -22 => return Err(VfsError::InvalidInput),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(readlinkout)
    }

    /// FuseSymlink = 6
    pub fn symlink(&self, name: &RelPath, link: &RelPath) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node SYMLINK({:?}) {:?} link to {:?} here...",
            FuseOpcode::FuseSymlink as u32,
            name,
            link
        );

        let symlink_error;
        let symlinkout;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let name_len = name.len();
        let link_len = link.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            42 + (name_len + link_len) as u32,
            FuseOpcode::FuseSymlink as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 280];
        fusein.write_to(&mut fusebuf);
        fusebuf[40..40 + name_len].copy_from_slice(name.as_bytes());
        fusebuf[41 + name_len..41 + name_len + link_len].copy_from_slice(link.as_bytes());
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at symlink in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseSymlink as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at symlink is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 144];
        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to symlink: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);

        if fuseout.is_ok() {
            symlinkout = FuseEntryOut::read_from(&outbuf[16..]);
            symlinkout.print();
            symlink_error = 1;
        } else {
            symlink_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node symlink finish successfully...");

        if symlink_error < 0 {
            match symlink_error {
                -13 => return Err(VfsError::PermissionDenied),
                -17 => return Err(VfsError::AlreadyExists),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseMknod = 8
    pub fn mknod(&self, name: &RelPath, ty: VfsNodeType) -> VfsResult {
        let newtype = match ty {
            VfsNodeType::Fifo => "fifo",
            VfsNodeType::CharDevice => "char device",
            VfsNodeType::BlockDevice => "block device",
            VfsNodeType::SymLink => "symlink",
            VfsNodeType::Socket => "socket",
            _ => "unknown",
        };
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node MKNOD({:?}) {:?} here, type: {:?}...",
            FuseOpcode::FuseMknod as u32,
            name,
            newtype
        );

        // panic!("fuse_node mknod not implemented yet...");
        let mknod_error;
        let mknodout;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let name_len = name.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            57 + name_len as u32,
            FuseOpcode::FuseMknod as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf);

        // char c 10 0:     mode: 0x21a4, rdev: 0xa00, umask: 18
        // block b 8 0:     mode: 0x61a4, rdev: 0x800, umask: 18
        // fifo p:          mode: 0x11a4, rdev: 0x0, umask: 18
        // socket s
        // rdev = majonr << 8 | minor
        let mode = match ty {
            VfsNodeType::Fifo => 0x11a4,
            VfsNodeType::CharDevice => 0x21a4,
            VfsNodeType::BlockDevice => 0x61a4,
            VfsNodeType::Socket => 0x81a4,
            _ => 0x21a4,
        };
        let rdev = 0xa00; // major << 8 | minor;
        let mknodin = FuseMknodIn::new(mode, rdev, 18);
        mknodin.write_to(&mut fusebuf[40..]);
        fusebuf[56..56 + name_len].copy_from_slice(name.as_bytes());
        fusein.print();
        mknodin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at mknod in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseMknod as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at mknod is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 144];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to mknod: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);

        if fuseout.is_ok() {
            mknodout = FuseEntryOut::read_from(&outbuf[16..]);
            mknodout.print();
            mknod_error = 1;
        } else {
            mknod_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node mknod finish successfully...");

        if mknod_error < 0 {
            match mknod_error {
                -13 => return Err(VfsError::PermissionDenied),
                -17 => return Err(VfsError::AlreadyExists),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseMkdir = 9
    pub fn mkdir(&self, name: &RelPath) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node MKDIR({:?}) {:?} here...",
            FuseOpcode::FuseMkdir as u32,
            name
        );

        let mkdir_error;
        let mkdirout;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let name_len = name.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            49 + name_len as u32,
            FuseOpcode::FuseMkdir as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf);
        let mkdirin = FuseMkdirIn::new(0x1ff, 18); // 0x1ed => 0755
        mkdirin.write_to(&mut fusebuf[40..]);
        fusebuf[48..48 + name_len].copy_from_slice(name.as_bytes());
        fusein.print();
        mkdirin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at mkdir in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseMkdir as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at mkdir is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 144];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to mkdir: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            mkdirout = FuseEntryOut::read_from(&outbuf[16..]);
            mkdirout.print();
            mkdir_error = 1;
        } else {
            mkdir_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node mkdir finish successfully...");

        if mkdir_error < 0 {
            match mkdir_error {
                -5 => return Err(VfsError::Io),
                -13 => return Err(VfsError::PermissionDenied),
                -17 => return Err(VfsError::AlreadyExists),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseRmdir = 11
    pub fn rmdir(&self, name: &RelPath) -> VfsResult {
        let mut dirents: [VfsDirEntry; 8] = [VfsDirEntry::new("", VfsNodeType::File); 8];
        let mut cur = 0;
        let node = self.try_get(&RelPath::new(name))?;
        node.get_attr()?;
        node.open()?;
        let mut num = node.read_dir(0, &mut dirents)?;
        while num > 0 {
            for entry in dirents.iter().take(num) {
                let name = String::from_utf8_lossy(entry.name_as_bytes()).to_string();
                if name == "." || name == ".." {
                    continue;
                }
                if name.starts_with(".") {
                    continue;
                }
                debug!("dirent name = {name:?}");
                let node_name = name.as_str();
                node.unlink(&RelPath::new(node_name))?;
            }
            cur += num;
            num = node.read_dir(cur, &mut dirents)?;
        }
        node.release()?;

        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node RMDIR({:?}) {:?} here...",
            FuseOpcode::FuseRmdir as u32,
            name
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let name_len = name.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            41 + name_len as u32,
            FuseOpcode::FuseRmdir as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf);
        fusebuf[40..40 + name_len].copy_from_slice(name.as_bytes());
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at rmdir in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseRmdir as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at rmdir is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to rmdir: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let rmdir_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node rmdir finish successfully...");

        if rmdir_error < 0 {
            match rmdir_error {
                -2 => return Err(VfsError::NotFound),
                -13 => return Err(VfsError::PermissionDenied),
                -20 => return Err(VfsError::NotADirectory),
                -38 => return Err(VfsError::FunctionNotImplemented),
                -39 => return Err(VfsError::DirectoryNotEmpty),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseUnlink = 10
    pub fn unlink_node(&self, name: &RelPath) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node UNLINK({:?}) {:?} here...",
            FuseOpcode::FuseUnlink as u32,
            name
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let name_len = name.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            41 + name_len as u32,
            FuseOpcode::FuseUnlink as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf);
        fusebuf[40..40 + name_len].copy_from_slice(name.as_bytes());
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at unlink in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseUnlink as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at unlink is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to unlink: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let unlink_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node unlink finish successfully...");

        if unlink_error < 0 {
            match unlink_error {
                -2 => return Err(VfsError::NotFound),
                -13 => return Err(VfsError::PermissionDenied),
                -20 => return Err(VfsError::NotADirectory),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseRead = 15
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node READ({:?}) here, offset: {:?}, buf_len: {:?}...",
            FuseOpcode::FuseRead as u32,
            offset,
            buf.len()
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();

        let fusein = FuseInHeader::new(
            80,
            FuseOpcode::FuseRead as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 80];
        fusein.write_to(&mut fusebuf);

        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );
        let mut flags_guard = self.flags.lock();
        let readflags = &mut *flags_guard;
        *readflags = 0x8002;
        let readsize = buf.len().min(4096) as u32;
        let readin = FuseReadIn::new(fh, offset, readsize, 0, 0, 0x8002);
        readin.write_to(&mut fusebuf[40..]);
        fusein.print();
        readin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at read in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseRead as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at read is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 70000];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to read: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        let outlen = vec.len() - 16;
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let read_error = if fuseout.is_ok() {
            let readout = &outbuf[16..outlen + 16];
            buf[..outlen].copy_from_slice(readout);
            debug!("readout_len: {outlen:?}");
            trace!("readout: {readout:?}");
            1
        } else {
            fuseout.error()
        };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node read len: {outlen:?} finish successfully...");

        if read_error < 0 {
            match read_error {
                -13 => Err(VfsError::PermissionDenied),
                -21 => Err(VfsError::IsADirectory),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(outlen)
        }
    }

    /// FuseStatfs = 17
    pub fn statfs(&self) -> VfsResult<FuseStatfsOut> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node STATFS({:?}) here...",
            FuseOpcode::FuseStatfs as u32
        );

        let statfs_error;
        let mut statfsout = FuseStatfsOut::default();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            40,
            FuseOpcode::FuseStatfs as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 40];
        fusein.write_to(&mut fusebuf);
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at statfs in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseStatfs as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at statfs is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 96];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to statfs: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            statfsout = FuseStatfsOut::read_from(&outbuf[16..]);
            statfsout.print();
            statfs_error = 1;
        } else {
            statfs_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node statfs finish successfully...");

        if statfs_error < 0 {
            match statfs_error {
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(statfsout)
    }

    /// FuseFlush = 25
    pub fn flush(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node FLUSH({:?}) here...",
            FuseOpcode::FuseFlush as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            64,
            FuseOpcode::FuseFlush as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 64];
        fusein.write_to(&mut fusebuf);
        let flushin = FuseFlushIn::new(fh, 0, 0, 0);
        flushin.write_to(&mut fusebuf[40..]);
        fusein.print();
        flushin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at flush in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseFlush as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at flush is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to flush: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let flush_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node flush finish successfully...");

        if flush_error < 0 {
            match flush_error {
                -13 => Err(VfsError::PermissionDenied),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(())
        }
    }

    /// FuseAccess = 34
    pub fn access(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node ACCESS({:?}) here...",
            FuseOpcode::FuseAccess as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            48,
            FuseOpcode::FuseAccess as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 48];
        fusein.write_to(&mut fusebuf);
        let accessin = FuseAccessIn::new(1);
        accessin.write_to(&mut fusebuf[40..]);
        fusein.print();
        accessin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at access in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseAccess as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at access is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to access: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let access_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node access finish successfully...");

        if access_error < 0 {
            match access_error {
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseRename2 = 45
    pub fn rename2(&self, old: &RelPath, new: &RelPath) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node RENAME2({:?}) from {:?} to {:?} here...",
            FuseOpcode::FuseRename2 as u32,
            old,
            new
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let old_len = old.len();
        let new_len = new.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            58 + (old_len + new_len) as u32,
            FuseOpcode::FuseRename2 as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 280];
        fusein.write_to(&mut fusebuf);
        let rename2in = FuseRename2In::new(1, 1);
        rename2in.write_to(&mut fusebuf[40..]);
        fusebuf[56..56 + old_len].copy_from_slice(old.as_bytes());
        fusebuf[57 + old_len..57 + old_len + new_len].copy_from_slice(new.as_bytes());
        fusein.print();
        rename2in.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at rename2 in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseRename2 as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at rename2 is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to rename2: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let rename_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node rename2 from {old:?} to {new:?} finish successfully...");

        if rename_error < 0 {
            match rename_error {
                -2 => return Err(VfsError::NotFound),
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    /// FuseLseek = 46
    pub fn lseek(&self, offset: u64, whence: u32) -> VfsResult<u64> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node LSEEK({:?}) offset: {:?}, whence: {:?} here...",
            FuseOpcode::FuseLseek as u32,
            offset,
            whence
        );

        let lseek_error;
        let mut lseekout = 0;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            64,
            FuseOpcode::FuseLseek as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 64];
        fusein.write_to(&mut fusebuf);
        let lseekin = FuseLseekIn::new(fh, offset, whence);
        lseekin.write_to(&mut fusebuf[40..]);
        fusein.print();
        lseekin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at lseek in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseLseek as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at lseek is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 24];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to lseek: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            lseekout = FuseLseekOut::read_from(&outbuf[16..]).get_offset();
            debug!("lseekout = {lseekout:?}");
            lseek_error = 1;
        } else {
            lseek_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node lseek finish successfully...");

        if lseek_error < 0 {
            match lseek_error {
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(lseekout)
    }

    /// FuseDestroy = 38
    pub fn destroy(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node DESTROY({:?}) here...",
            FuseOpcode::FuseDestroy as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            40,
            FuseOpcode::FuseDestroy as u32,
            unique_id,
            1,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 40];
        fusein.write_to(&mut fusebuf);
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at destroy in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseDestroy as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at destroy is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to destroy: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let destroy_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node destroy finish successfully...");

        if destroy_error < 0 {
            match destroy_error {
                -5 => return Err(VfsError::Io),
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        UNIQUE_ID.store(0, Ordering::Relaxed);
        INITFLAG.store(1, Ordering::Relaxed);
        FUSE_VEC.lock().clear();

        Ok(())
    }
}

impl VfsNodeOps for FuseNode {
    /// FuseOpen = 14
    fn open(&self) -> Result<Option<Arc<dyn VfsNodeOps>>, VfsError> {
        if self.is_dir() {
            return self.open_dir();
        }

        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node OPEN({:?}) here...",
            FuseOpcode::FuseOpen as u32
        );

        let open_error;
        let mut openout = FuseOpenOut::default();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        let mut flags = self.file_flags();
        if flags == 0x8001 {
            flags = 0x8002;
        }
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}, flags: {:#x}",
            pid,
            nodeid,
            fh,
            self.is_dir(),
            flags
        );

        let fusein = FuseInHeader::new(
            48,
            FuseOpcode::FuseOpen as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 48];
        fusein.write_to(&mut fusebuf);
        let openin = FuseOpenIn::new(flags, 0);
        openin.write_to(&mut fusebuf[40..]);
        fusein.print();
        openin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at open in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseOpen as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at open is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 32];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to open: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();
        if fuseout.is_ok() {
            openout = FuseOpenOut::read_from(&outbuf[16..]);
            openout.print();
            open_error = 1;
        } else {
            open_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node open finish successfully...");

        if open_error < 0 {
            match open_error {
                -13 => return Err(VfsError::PermissionDenied),
                -21 => return Err(VfsError::IsADirectory),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        self.set_fh(openout.get_fh());
        debug!(
            "fh = {:#x}, openout.fh = {:#x}",
            self.get_fh(),
            openout.get_fh()
        );

        Ok(None)
    }

    /// FuseRelease = 18
    fn release(&self) -> VfsResult {
        if self.is_dir() {
            return self.release_dir();
        }

        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node RELEASE({:?}) here...",
            FuseOpcode::FuseRelease as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        let flags = self.get_node_flags();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}, flags: {:#x}",
            pid,
            nodeid,
            fh,
            self.is_dir(),
            flags
        );

        let fusein = FuseInHeader::new(
            64,
            FuseOpcode::FuseRelease as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 64];
        fusein.write_to(&mut fusebuf);
        let releasein = FuseReleaseIn::new(fh, flags, 0, 0);
        releasein.write_to(&mut fusebuf[40..]);
        fusein.print();
        releasein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at release in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseRelease as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at release is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to release: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let release_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node release finish successfully...");

        if release_error < 0 {
            match release_error {
                -13 => Err(VfsError::PermissionDenied),
                -21 => Err(VfsError::IsADirectory),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(())
        }
    }

    /// FuseGetattr = 3
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.check_init();
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node GET_ATTR({:?}) here...",
            FuseOpcode::FuseGetattr as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            56,
            FuseOpcode::FuseGetattr as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 56];
        fusein.write_to(&mut fusebuf);
        let getattrin = FuseGetattrIn::new(0, 0, fh);
        getattrin.write_to(&mut fusebuf[40..]);
        fusein.print();
        getattrin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at get_attr in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseGetattr as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at get_attr is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 120];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to get_attr: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();
        let fuseattr = FuseAttrOut::read_from(&outbuf[16..]);
        fuseattr.print();

        self.set_node_attr(fuseattr.get_attr());
        let attr_size = fuseattr.get_size();

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node get_attr finish successfully...");

        if self.is_dir() {
            Ok(VfsNodeAttr::new_dir(self.get_node_inode(), attr_size, 0))
        } else {
            Ok(VfsNodeAttr::new_file(self.get_node_inode(), attr_size, 0))
        }
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.try_get(&RelPath::new("..")).ok()
    }

    fn get_inode(&self) -> Option<u64> {
        let curid = self.get_node_inode();
        Some(curid)
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        debug!(
            "\nFUSE READ AT here, offset: {:?}, buf_len: {:?}\n",
            offset,
            buf.len()
        );
        let mut remain = buf.len();
        let mut cur_offset = offset;
        let mut start = 0;
        while remain > 0 {
            let cur = remain.min(4096);
            let read_len = self.read(cur_offset, &mut buf[start..start + cur])?;
            cur_offset += read_len as u64;
            start += read_len;
            remain -= read_len;
            if read_len < cur {
                break;
            }
        }

        Ok(start)
    }

    /// FuseWrite = 16
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node WRITE({:?}) here, offset: {:?}, buf_len: {:?}",
            FuseOpcode::FuseWrite as u32,
            offset,
            buf.len()
        );
        trace!("buf: {buf:?}");

        let write_error;
        let writeout;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let buf_len = buf.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        let flags = self.file_flags();
        // let mut flags_guard = self.flags.lock();
        // let wflags = &mut *flags_guard;
        // *wflags = flags;
        self.set_node_flags(flags);
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}, flags: {:#x}",
            pid,
            nodeid,
            fh,
            self.is_dir(),
            flags
        );

        let fusein = FuseInHeader::new(
            80 + buf_len as u32,
            FuseOpcode::FuseWrite as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 72000];
        fusein.write_to(&mut fusebuf);
        let writein = FuseWriteIn::new(fh, offset, (buf_len + 1) as u32, 0, 0, flags);
        writein.write_to(&mut fusebuf[40..]);
        fusebuf[80..80 + buf_len].copy_from_slice(buf);
        fusein.print();
        writein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at write in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseWrite as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at write is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 24];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to write: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            writeout = FuseWriteOut::read_from(&outbuf[16..]);
            writeout.print();
            write_error = 1;
        } else {
            write_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node write finish successfully...");

        if write_error < 0 {
            match write_error {
                -2 => Err(VfsError::NotFound),
                -13 => Err(VfsError::PermissionDenied),
                -21 => Err(VfsError::IsADirectory),
                -28 => Err(VfsError::StorageFull),
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(buf.len())
        }
    }

    /// FuseFsync = 20
    fn fsync(&self) -> VfsResult {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node FSYNC({:?}) here...",
            FuseOpcode::FuseFsync as u32
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            40,
            FuseOpcode::FuseFsync as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 40];
        fusein.write_to(&mut fusebuf);
        fusein.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at fsync in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseFsync as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at fsync is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to fsync: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let fsync_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node fsync finish successfully...");

        if fsync_error < 0 {
            match fsync_error {
                -38 => Err(VfsError::FunctionNotImplemented),
                _ => Err(VfsError::PermissionDenied),
            }
        } else {
            Ok(())
        }
    }

    fn truncate(&self, size: u64) -> VfsResult {
        warn!("fuse_node truncate is not implemented, size: {size:?}...");
        Ok(())
    }

    fn lookup(self: Arc<Self>, raw_path: &RelPath) -> VfsResult<VfsNodeRef> {
        if raw_path.as_str() == "MAILPATH" {
            return Err(VfsError::NotFound);
        }
        self.try_get(raw_path)
    }

    /// FuseCreate = 20
    fn create(&self, path: &RelPath, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        let (name, raw_rest) = split_path(path.as_str());
        if let Some(rest) = raw_rest {
            if name.is_empty() || name == "." {
                return self.create(&RelPath::new(rest), ty, mode);
            }
            return self
                .try_get(&RelPath::new(name))?
                .create(&RelPath::new(rest), ty, mode);
        }

        if ty == VfsNodeType::Dir {
            return self.mkdir(path);
        } else if ty != VfsNodeType::File {
            return self.mknod(path, ty);
        }

        let newtype = match ty {
            VfsNodeType::Fifo => "fifo",
            VfsNodeType::CharDevice => "char device",
            VfsNodeType::BlockDevice => "block device",
            VfsNodeType::File => "file",
            VfsNodeType::SymLink => "symlink",
            VfsNodeType::Socket => "socket",
            _ => "unknown",
        };
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node CREATE({:?}) {:?} here, type: {:?}...",
            FuseOpcode::FuseCreate as u32,
            path,
            newtype
        );

        let create_error;
        let createout;
        let openout;

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let path_len = path.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            57 + path_len as u32,
            FuseOpcode::FuseCreate as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 180];
        fusein.write_to(&mut fusebuf);
        let createin = FuseCreateIn::new(0x8241, 0x81a4, 18, 0);
        createin.write_to(&mut fusebuf[40..]);
        fusebuf[56..56 + path_len].copy_from_slice(path.as_bytes());
        fusein.print();
        createin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at create in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseCreate as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at create is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 160];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to create: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        if fuseout.is_ok() {
            createout = FuseEntryOut::read_from(&outbuf[16..]);
            createout.print();
            openout = FuseOpenOut::read_from(&outbuf[144..]);
            openout.print();
            create_error = 1;
        } else {
            create_error = fuseout.error();
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node create finish successfully...");

        if create_error < 0 {
            match create_error {
                -5 => return Err(VfsError::Io),
                -13 => return Err(VfsError::PermissionDenied),
                -17 => return Err(VfsError::AlreadyExists),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }

    fn unlink(&self, path: &RelPath) -> VfsResult {
        let (name, raw_rest) = split_path(path);
        if let Some(rest) = raw_rest {
            if name.is_empty() || name == "." {
                return self.unlink(&RelPath::new(rest));
            }
            return self
                .try_get(&RelPath::new(name))?
                .unlink(&RelPath::new(rest));
        }

        let node = self.try_get(&RelPath::new(name))?;
        let attr = node.get_attr()?;

        if attr.file_type() == VfsNodeType::Dir {
            self.rmdir(&RelPath::new(name))
        } else {
            self.unlink_node(&RelPath::new(name))
        }
    }

    /// FuseReaddir = 28
    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node READ_DIR({:?}) here, start: {:?}...",
            FuseOpcode::FuseReaddir as u32,
            start_idx
        );

        let mut dirs = Vec::<FuseDirent>::new();

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );

        let fusein = FuseInHeader::new(
            80,
            FuseOpcode::FuseReaddir as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 80];
        fusein.write_to(&mut fusebuf);
        let readin = FuseReadIn::new(fh, 0, 4096, 0, 0, 0x18800);
        readin.write_to(&mut fusebuf[40..]);
        fusein.print();
        readin.print();

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at readdir in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseReaddir as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at readdir is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
            // ruxtask::WaitQueue
        }

        let mut outbuf = [0; 12000];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to readdir: {vec:?}");
        let buf_len = vec.len();
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let readdir_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        let mut offset = 16;
        while offset < buf_len {
            let direntry = FuseDirent::read_from(&outbuf[offset..]);
            direntry.print();
            offset += direntry.get_len();
            dirs.push(direntry);
            debug!("offset = {offset:?}");
        }

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node readdir finish successfully...");

        if readdir_error < 0 {
            match readdir_error {
                -13 => return Err(VfsError::PermissionDenied),
                -20 => return Err(VfsError::NotADirectory),
                -22 => return Err(VfsError::InvalidInput),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    trace!("dirs.len() = {:?}, i+idx = {:?}", dirs.len(), i + start_idx);
                    if let Some(entry) = dirs.get(i + start_idx) {
                        let entry_type = entry.get_type_as_vfsnodetype();
                        *ent = VfsDirEntry::new(&entry.get_name(), entry_type);
                        trace!(
                            "entry: {{ name: {:?}, type: {:?} }}",
                            &entry.get_name(),
                            entry_type
                        );
                    } else {
                        for (j, dirent) in dirents.iter().enumerate().take(i) {
                            debug!(
                                "entry {:?}: name: {:?}, type: {:?}",
                                j,
                                String::from_utf8(dirent.name_as_bytes().to_vec()),
                                &dirents[j].entry_type()
                            );
                        }
                        debug!("Ok(i) = {i:?}");
                        return Ok(i);
                    }
                }
            }
        }

        for (j, dirent) in dirents.iter().enumerate() {
            debug!(
                "entry {:?}: name: {:?}, type: {:?}",
                j,
                String::from_utf8(dirent.name_as_bytes().to_vec()),
                &dirents[j].entry_type()
            );
        }

        debug!("Ok(dirents.len()) = {:?}", dirents.len());
        Ok(dirents.len())
    }

    /// FuseRename = 12
    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult {
        debug!(
            "fuse_node(inode: {:?}) rename src: {:?}, dst: {:?}",
            self.get_node_inode(),
            src_path,
            dst_path
        );

        if NEWID.load(Ordering::SeqCst) == -1 {
            NEWID.store(self.find_inode(dst_path).unwrap() as i32, Ordering::Relaxed);
        }
        let newid = NEWID.load(Ordering::SeqCst);

        let (src_name, src_rest1) = split_path(src_path);
        if let Some(src_rest) = src_rest1 {
            if src_name.is_empty() || src_name == "." {
                return self.rename(&RelPath::new(src_rest), dst_path);
            }
            return self
                .try_get(&RelPath::new(src_name))?
                .rename(&RelPath::new(src_rest), dst_path);
        }

        // let newid = self.find_inode(dst_path).unwrap();
        let raw_dst_name = self.get_final_name(dst_path).unwrap();
        let dst_name = raw_dst_name.as_str();

        // self.rename2(src_path, dst_path);

        debug!(
            "\nNEW FUSE REQUEST:\n  fuse_node RENAME({:?}) from {:?} to {:?} here...",
            FuseOpcode::FuseRename as u32,
            src_path,
            dst_path
        );

        UNIQUE_ID.fetch_add(2, Ordering::Relaxed);
        let unique_id = UNIQUE_ID.load(Ordering::SeqCst);
        let pid = current().id().as_u64();
        let src_len = src_name.len();
        let dst_len = dst_name.len();
        let nodeid = self.get_node_inode();
        let fh = self.get_fh();
        debug!(
            "pid = {:?}, inode = {:?}, fh = {:#x}, is_dir: {:?}",
            pid,
            nodeid,
            fh,
            self.is_dir()
        );
        debug!(
            "src_name = {src_name:?}, dst_name = {dst_name:?}, src_len = {src_len:?}, dst_len = {dst_len:?}"
        );

        let fusein = FuseInHeader::new(
            50 + (src_len + dst_len) as u32,
            FuseOpcode::FuseRename as u32,
            unique_id,
            nodeid,
            1000,
            1000,
            pid as u32,
        );
        let mut fusebuf = [0; 280];
        fusein.write_to(&mut fusebuf);
        debug!("oldid = {nodeid:?}, newid = {newid:?}");
        let renamein = FuseRenameIn::new(newid as u64);
        renamein.write_to(&mut fusebuf[40..]);
        fusebuf[48..48 + src_len].copy_from_slice(src_name.as_bytes());
        fusebuf[49 + src_len..49 + src_len + dst_len].copy_from_slice(dst_name.as_bytes());
        fusein.print();
        renamein.print();
        NEWID.store(-1, Ordering::Relaxed);

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(&fusebuf);
        trace!("Fusevec at rename in devfuse: {vec:?}");

        FUSEFLAG.store(FuseOpcode::FuseRename as i32, Ordering::Relaxed);

        loop {
            let flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag < 0 {
                trace!("Fuseflag at rename is set to {flag:?}, exiting loop. !!!");
                break;
            }
            ruxtask::yield_now();
        }

        let mut outbuf = [0; 16];

        let mut vec = FUSE_VEC.lock();
        trace!("Fusevec back to rename: {vec:?}");
        outbuf[0..vec.len()].copy_from_slice(&vec);
        vec.clear();

        let fuseout = FuseOutHeader::read_from(&outbuf);
        fuseout.print();

        let rename_error = if fuseout.is_ok() { 1 } else { fuseout.error() };

        FUSEFLAG.store(0, Ordering::Relaxed);

        debug!("fuse_node rename from {src_path:?} to {dst_path:?} finish successfully...");

        if rename_error < 0 {
            match rename_error {
                -2 => return Err(VfsError::NotFound),
                -13 => return Err(VfsError::PermissionDenied),
                -38 => return Err(VfsError::FunctionNotImplemented),
                _ => return Err(VfsError::PermissionDenied),
            }
        }

        Ok(())
    }
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

/// Create a new FuseFS instance
pub fn fusefs() -> Arc<FuseFS> {
    trace!("fusefs newfs here...");
    Arc::new(FuseFS::new())
}
