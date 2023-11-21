/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! 9P filesystem used by [RukOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is based on [`axfs_vfs`].
use crate::drv::{self, Drv9pOps};
use alloc::{collections::BTreeMap, string::String, string::ToString, sync::Arc, sync::Weak};
use axfs_vfs::{
    impl_vfs_non_dir_default, VfsDirEntry, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodeRef,
    VfsNodeType, VfsOps, VfsResult,
};
use log::*;
use spin::{once::Once, RwLock};

macro_rules! handle_result {
    ($result:expr, $error_msg:expr) => {
        match $result {
            Ok(_) => {
                // Handle the Ok case
            }
            Err(err_code) => {
                error!($error_msg, err_code);
            }
        }
    };
}

/// A 9P filesystem that implements [`axfs_vfs::VfsOps`].
pub struct _9pFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
}

impl _9pFileSystem {
    /// Create a new instance.
    pub fn new(dev: Arc<RwLock<Drv9pOps>>, aname: &str, protocol: &str) -> Self {
        // Initialize 9pfs version to make sure protocol is right.
        // Select 9P2000.L at defealt first trial.
        let mut protocol = protocol.to_string();
        match dev.write().tversion(&protocol) {
            Ok(protocol_server) => {
                info!("9pfs server's protocol: {}", protocol_server);
                if protocol != protocol_server {
                    protocol = protocol_server;
                }
            }
            Err(errcode) => {
                error!("9pfs tversion failed! error code: {}", errcode);
            }
        }

        // root_fid should never be recycled.
        let fid = match dev.write().get_fid() {
            Some(id) => id,
            None => {
                warn!("9pfs: No enough fids! Check fid_MAX constrant or fid leaky.");
                0xff_ff_ff_ff
            }
        };

        const AFID: u32 = 0xFFFF_FFFF;

        // AUTH afid
        #[cfg(feature = "need_auth")]
        handle_result!(
            dev.write().tauth(AFID, "rukos", "/"),
            "9pfs auth failed! error code: {}"
        );

        // attach dir to fid
        handle_result!(
            dev.write().tattach(fid, AFID, "rukos", aname),
            "9pfs attach failed! error code: {}"
        );

        Self {
            parent: Once::new(),
            root: DirNode::new(None, dev.clone(), fid, Arc::new(protocol.clone())),
        }
    }
}

impl VfsOps for _9pFileSystem {
    fn mount(&self, _path: &str, mount_point: VfsNodeRef) -> VfsResult {
        if let Some(parent) = mount_point.parent() {
            self.root.set_parent(Some(self.parent.call_once(|| parent)));
        } else {
            self.root.set_parent(None);
        }
        Ok(())
    }

    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

/// The directory node in the 9P filesystem.
///
/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct DirNode {
    inner: Arc<RwLock<Drv9pOps>>,
    this: Weak<DirNode>,
    fid: Arc<u32>,
    protocol: Arc<String>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
}

impl DirNode {
    pub(super) fn new(
        parent: Option<Weak<dyn VfsNodeOps>>,
        dev: Arc<RwLock<Drv9pOps>>,
        fid: u32,
        protocol: Arc<String>,
    ) -> Arc<Self> {
        if *protocol == "9P2000.L" {
            match dev.write().l_topen(fid, 0x00) {
                Ok(_) => {}
                Err(errcode) => {
                    error!("9pfs open failed! error code: {}", errcode);
                }
            }
        } else if *protocol == "9P2000.u" {
            match dev.write().topen(fid, 0x00) {
                Ok(_) => {}
                Err(errcode) => {
                    error!("9pfs open failed! error code: {}", errcode);
                }
            }
        } else {
            error!("9pfs open failed! Unsupported protocol version");
        }

        Arc::new_cyclic(|this| Self {
            inner: dev,
            this: this.clone(),
            fid: Arc::new(fid),
            protocol,
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Checks whether a node with the given name exists in this directory.
    pub fn exist(&self, name: &str) -> bool {
        debug!("finding if {} exists", name);
        match self.read_nodes() {
            Ok(map) => map.contains_key(name),
            Err(_) => false,
        }
    }

    /// Creates a new node with the given name and type in this directory.
    pub fn create_node(&self, name: &str, ty: VfsNodeType) -> VfsResult {
        if self.exist(name) {
            error!("AlreadyExists {}", name);
            return Err(VfsError::AlreadyExists);
        }
        let fid = match self.inner.write().get_fid() {
            Some(id) => id,
            None => {
                error!("No enough fids! Check fid_MAX constrant or fid leaky.");
                0xff_ff_ff_ff
            }
        };
        match ty {
            VfsNodeType::File => {
                handle_result!(
                    self.inner.write().twalk(*self.fid, fid, 0, &[]),
                    "9pfs twalk failed! error code: {}"
                );
                if *self.protocol == "9P2000.L" {
                    handle_result!(
                        self.inner.write().l_tcreate(fid, name, 0x02, 0o100644, 500),
                        "9pfs l_create failed! error code: {}"
                    );
                } else if *self.protocol == "9P2000.u" {
                    handle_result!(
                        self.inner.write().u_tcreate(fid, name, 0o777, 0o02, ""),
                        "9pfs create failed! error code: {}"
                    );
                } else {
                    return Err(VfsError::Unsupported);
                }
            }
            VfsNodeType::Dir => {
                handle_result!(
                    self.inner.write().tmkdir(*self.fid, name, 0o40755, 500),
                    "9pfs mkdir failed! error code: {}"
                );
                handle_result!(
                    self.inner.write().twalk(*self.fid, fid, 1, &[&name]),
                    "9pfs twalk failed! error code: {}"
                );
            }
            _ => return Err(VfsError::Unsupported),
        }

        handle_result!(
            self.inner.write().tclunk(fid),
            "9pfs tclunk failed! error code: {}"
        );
        self.inner.write().recycle_fid(fid);

        Ok(())
    }

    /// Removes a node by the given name in this directory.
    pub fn remove_node(&self, name: &str) -> VfsResult {
        if self.exist(name) {
            let fid = match self.inner.write().get_fid() {
                Some(id) => id,
                None => {
                    error!("No enough fids! Check fid_MAX constrant or fid leaky.");
                    0xff_ff_ff_ff
                }
            };
            handle_result!(
                self.inner.write().twalk(*self.fid, fid, 1, &[name]),
                "9pfs twalk failed! error code: {}"
            );
            handle_result!(
                self.inner.write().tremove(fid),
                "9pfs tremove failed! error code: {}"
            );
            self.inner.write().recycle_fid(fid);
            Ok(())
        } else {
            Err(VfsError::NotFound)
        }
    }

    /// Update nodes from host filesystem
    fn read_nodes(&self) -> Result<BTreeMap<String, VfsNodeRef>, VfsError> {
        let mut node_map: BTreeMap<String, VfsNodeRef> = BTreeMap::new();
        debug!("reading nodes");
        let dirents = match self.protocol.as_str() {
            "9P2000.L" => match self.inner.write().treaddir(*self.fid) {
                Ok(contents) => contents,
                Err(errcode) => {
                    error!("9pfs treaddir failed! error code: {}", errcode);
                    return Err(VfsError::BadState);
                }
            },
            "9P2000.u" => match self.inner.write().u_treaddir(*self.fid) {
                Ok(contents) => contents,
                Err(errcode) => {
                    error!("9pfs u_treaddir failed! error code: {}", errcode);
                    return Err(VfsError::BadState);
                }
            },
            _ => {
                error!("Unsupport 9P protocol version: {}", *self.protocol);
                return Err(VfsError::BadState);
            }
        };

        for direntry in dirents {
            let fname = direntry.get_name();
            if fname.eq("..") || fname.eq(".") {
                continue;
            }
            let ftype = match direntry.get_type() {
                0o1 => VfsNodeType::Fifo,
                0o2 => VfsNodeType::CharDevice,
                0o4 => VfsNodeType::Dir,
                0o6 => VfsNodeType::BlockDevice,
                0o10 => VfsNodeType::File,
                0o12 => VfsNodeType::SymLink,
                0o14 => VfsNodeType::Socket,
                _ => {
                    error!("Unsupported File Type In 9pfs! Using it as File");
                    VfsNodeType::File
                }
            };
            debug!("9pfs update node {}, type {}", fname, direntry.get_type());
            let fid = match self.inner.write().get_fid() {
                Some(id) => id,
                None => {
                    error!("No enough fids! Check fid_MAX constrant or fid leaky.");
                    return Err(VfsError::BadState);
                }
            };
            self.inner
                .write()
                .twalk(*self.fid, fid, 1, &[fname])
                .unwrap();
            let node: VfsNodeRef = match ftype {
                VfsNodeType::File => Arc::new(FileNode::new(
                    self.inner.clone(),
                    fid,
                    self.protocol.clone(),
                )),
                VfsNodeType::Dir => Self::new(
                    Some(self.this.clone()),
                    self.inner.clone(),
                    fid,
                    self.protocol.clone(),
                ),
                _ => return Err(VfsError::Unsupported),
            };
            node_map.insert(fname.into(), node);
        }
        Ok(node_map)
    }
}

impl Drop for DirNode {
    fn drop(&mut self) {
        // pay attention to AA-deadlock
        let result = self.inner.write().tclunk(*self.fid);
        match result {
            Ok(_) => {
                self.inner.write().recycle_fid(*self.fid);
            }
            Err(_) => {
                error!("9pfs drop failed! It may cause fid leaky problem.")
            }
        }
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        if *self.protocol == "9P2000.L" {
            let resp = self.inner.write().tgetattr(*self.fid, 0x3fff_u64);
            match resp {
                Ok(attr) => Ok(VfsNodeAttr::new_dir(attr.get_size(), attr.get_blk_num())),
                Err(_) => Err(VfsError::BadState),
            }
        } else if *self.protocol == "9P2000.u" {
            let resp = self.inner.write().tstat(*self.fid);
            match resp {
                Ok(stat) => Ok(VfsNodeAttr::new_dir(stat.get_length(), stat.get_blk_num())),
                Err(_) => Err(VfsError::BadState),
            }
        } else {
            Err(VfsError::Unsupported)
        }
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.parent.read().upgrade()
    }

    /// for 9p filesystem's directory, lookup() will return node in 9p if path existing in both 9p and mounted_map.
    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        debug!("9pfs lookup:{}, {:?}", name, rest);
        let _9p_map = match self.read_nodes() {
            Ok(contents) => contents,
            Err(errcode) => {
                error!("9pfs read_nodes failed! error code = {}", errcode);
                return Err(VfsError::Unsupported);
            }
        };

        // find file in 9p host first, and then find in host if failed
        let node = match _9p_map.get(name) {
            Some(node) => node.clone(),
            None => {
                debug!("find no {:?} in 9p dir", name);
                match name {
                    "" | "." => Ok(self.clone() as VfsNodeRef),
                    ".." => self.parent().ok_or(VfsError::NotFound),
                    _ => Err(VfsError::NotFound),
                }?
            }
        };

        if let Some(rest) = rest {
            node.lookup(rest)
        } else {
            Ok(node)
        }
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        debug!("9pfs reading dirents");
        let _9p_map = match self.read_nodes() {
            Ok(contents) => contents,
            Err(errcode) => {
                error!("9pfs read_nodes failed! error code = {}", errcode);
                return Err(VfsError::BadState);
            }
        };

        let mut item_iter = _9p_map.iter().skip(start_idx.max(2) - 2);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some((name, node)) = item_iter.next() {
                        let attr = node.get_attr();
                        let file_type = match attr {
                            Ok(attr) => attr.file_type(),
                            Err(ecode) => {
                                error!("get [{}] attribute failed, error code:{}.", name, ecode);
                                continue;
                            }
                        };
                        *ent = VfsDirEntry::new(name, file_type);
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at 9pfs: {}", ty, path);
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.create(rest, ty),
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(rest, ty),
                _ => {
                    let subdir = self
                        .read_nodes()?
                        .get(name)
                        .ok_or(VfsError::NotFound)?
                        .clone();
                    subdir.create(rest, ty)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Ok(()) // already exists
        } else {
            self.create_node(name, ty)
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove at 9pfs: {}", path);
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.remove(rest),
                ".." => self.parent().ok_or(VfsError::NotFound)?.remove(rest),
                _ => {
                    let subdir = match self.read_nodes() {
                        Ok(contents) => contents.get(name).ok_or(VfsError::NotFound)?.clone(),
                        Err(errcode) => {
                            error!("9pfs read_nodes failed! error code = {}", errcode);
                            return Err(VfsError::BadState);
                        }
                    };
                    subdir.remove(rest)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Err(VfsError::InvalidInput) // remove '.' or '..
        } else {
            self.remove_node(name)
        }
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

/// The file node in the 9P filesystem.
///
/// It implements [`axfs_vfs::VfsNodeOps`].
/// Note: Pay attention to AA-deadlock in inner.
pub struct FileNode {
    inner: Arc<RwLock<Drv9pOps>>,
    fid: Arc<u32>,
    protocol: Arc<String>,
}

impl FileNode {
    pub(super) fn new(dev: Arc<RwLock<Drv9pOps>>, fid: u32, protocol: Arc<String>) -> Self {
        const OPEN_FLAG: u32 = 0x02;
        if *protocol == "9P2000.L" {
            handle_result!(
                dev.write().l_topen(fid, OPEN_FLAG),
                "9pfs l_topen failed! error code: {}"
            );
        } else if *protocol == "9P2000.u" {
            handle_result!(
                dev.write().topen(fid, OPEN_FLAG as u8),
                "9pfs topen failed! error code: {}"
            );
        }
        Self {
            inner: dev,
            fid: Arc::new(fid),
            protocol,
        }
    }
}

impl Drop for FileNode {
    fn drop(&mut self) {
        // pay attention to AA-deadlock
        let result = self.inner.write().tclunk(*self.fid);
        match result {
            Ok(_) => {
                self.inner.write().recycle_fid(*self.fid);
            }
            Err(_) => {
                error!("9pfs drop failed! It may cause fid leaky problem.")
            }
        }
    }
}

impl VfsNodeOps for FileNode {
    impl_vfs_non_dir_default! {}
    /// Get the attributes of the node.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        if *self.protocol == "9P2000.L" {
            let resp = self.inner.write().tgetattr(*self.fid, 0x3fff_u64);
            match resp {
                Ok(attr) => Ok(VfsNodeAttr::new_file(attr.get_size(), attr.get_blk_num())),
                Err(_) => Err(VfsError::BadState),
            }
        } else if *self.protocol == "9P2000.u" {
            let resp = self.inner.write().tstat(*self.fid);
            match resp {
                Ok(stat) => Ok(VfsNodeAttr::new_file(stat.get_length(), stat.get_blk_num())),
                Err(_) => Err(VfsError::BadState),
            }
        } else {
            Err(VfsError::Unsupported)
        }
    }

    /// Truncate the file to the given size.
    fn truncate(&self, size: u64) -> VfsResult {
        debug!("9pfs truncating, size:{}", size);
        if *self.protocol == "9P2000.L" {
            let mut attr = drv::FileAttr::new();
            attr.set_size(size);
            match self.inner.write().tsetattr(*self.fid, attr) {
                Ok(_) => Ok(()),
                Err(_) => Err(VfsError::BadState),
            }
        } else if *self.protocol == "9P2000.u" {
            let resp = self.inner.write().tstat(*self.fid);
            let mut stat = match resp {
                Ok(state) => state,
                Err(_) => return Err(VfsError::BadState),
            };
            stat.set_length(size);
            match self.inner.write().twstat(*self.fid, stat) {
                Ok(_) => Ok(()),
                Err(_) => Err(VfsError::BadState),
            }
        } else {
            error!("{} is not supported", self.protocol);
            Err(VfsError::Unsupported)
        }
    }

    /// Read data from the file at the given offset.
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut dev = self.inner.write();
        let mut read_len = buf.len();
        let mut offset_ptr = 0;
        while read_len > 0 {
            let target_buf = &mut buf[offset_ptr..];
            let rlen = match dev.tread(*self.fid, offset + offset_ptr as u64, read_len as u32) {
                Ok(content) => {
                    let read_len = content.len();
                    target_buf[..read_len].copy_from_slice(&content);
                    read_len
                }
                Err(_) => return Err(VfsError::BadState),
            };
            if rlen == 0 {
                return Ok(offset_ptr);
            }
            read_len -= rlen;
            offset_ptr += rlen;
        }
        Ok(buf.len())
    }

    /// Write data to the file at the given offset.
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut dev = self.inner.write();
        let mut write_len = buf.len();
        let mut offset_ptr = 0;
        while write_len > 0 {
            let target_buf = &buf[offset_ptr..];
            let wlen = match dev.twrite(*self.fid, offset + offset_ptr as u64, target_buf) {
                Ok(writed_length) => writed_length as usize,
                Err(_) => return Err(VfsError::BadState),
            };
            if wlen == 0 {
                return Ok(offset_ptr);
            }
            write_len -= wlen;
            offset_ptr += wlen;
        }
        Ok(buf.len())
    }

    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> VfsResult {
        let mut dev = self.inner.write();
        match dev.tfsync(*self.fid) {
            Ok(_) => Ok(()),
            Err(_) => Err(VfsError::BadState),
        }
    }
}
