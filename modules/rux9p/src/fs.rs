/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! 9P filesystem used by [Ruxos](https://github.com/syswonder/ruxos).
//!
//! The implementation is based on [`axfs_vfs`].
use crate::drv::{self, Drv9pOps};
use alloc::{string::String, string::ToString, sync::Arc, sync::Weak, vec::Vec};
use axfs_vfs::{
    VfsDirEntry, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps,
    VfsResult,
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
    root: Arc<CommonNode>,
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

        const AFID: u32 = 0xFFFF_FFFF;

        let fid = match dev.write().get_fid() {
            Some(id) => id,
            None => {
                panic!("9pfs: No enough fids! Check fid_MAX constrant or fid leaky.");
            }
        };

        // AUTH afid
        #[cfg(feature = "need_auth")]
        handle_result!(
            dev.write().tauth(AFID, "ruxos", "/"),
            "9pfs auth failed! error code: {}"
        );

        // attach dir to fid
        handle_result!(
            dev.write().tattach(fid, AFID, "ruxos", aname),
            "9pfs attach failed! error code: {}"
        );

        Self {
            parent: Once::new(),
            root: CommonNode::new(fid, None, dev.clone(), Arc::new(protocol.clone())),
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
pub struct CommonNode {
    this: Weak<CommonNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    inner: Arc<RwLock<Drv9pOps>>,
    fid: Arc<u32>,
    protocol: Arc<String>,
}

impl CommonNode {
    pub(super) fn new(
        fid: u32,
        parent: Option<Weak<dyn VfsNodeOps>>,
        dev: Arc<RwLock<Drv9pOps>>,
        protocol: Arc<String>,
    ) -> Arc<Self> {
        const O_RDWR: u8 = 0x02;
        const O_RDONLY: u8 = 0x00;
        const EISDIR: u8 = 21;

        let result = if *protocol == "9P2000.L" {
            dev.write().l_topen(fid, O_RDWR as u32)
        } else if *protocol == "9P2000.u" {
            dev.write().topen(fid, O_RDWR)
        } else {
            error!("9pfs open failed! Unsupported protocol version");
            Ok(())
        };

        if let Err(EISDIR) = result {
            if *protocol == "9P2000.L" {
                handle_result!(
                    dev.write().l_topen(fid, O_RDONLY as u32),
                    "9pfs l_topen failed! error code: {}"
                );
            } else if *protocol == "9P2000.u" {
                handle_result!(
                    dev.write().topen(fid, O_RDONLY),
                    "9pfs topen failed! error code: {}"
                );
            } else {
                error!("9pfs open failed! Unsupported protocol version");
            }
        } else if let Err(ecode) = result {
            error!("9pfs topen failed! error code: {}", ecode);
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
    fn exist(&self, path: &str) -> bool {
        self.try_get(path).is_ok()
    }

    /// Creates a new node with the given name and type in this directory.
    fn create_node(&self, name: &str, ty: VfsNodeType) -> VfsResult {
        if self.exist(name) {
            error!("AlreadyExists {}", name);
            return Err(VfsError::AlreadyExists);
        }
        let fid = match self.inner.write().get_fid() {
            Some(id) => id,
            None => {
                panic!("9pfs: No enough fids! Check fid_MAX constrant or fid leaky.");
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

    fn try_get(&self, path: &str) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        if name == ".." {
            return self.parent().unwrap().lookup(rest.unwrap_or(""));
        } else if name == "." {
            return self.try_get(rest.unwrap_or(""));
        }

        let fid = match self.inner.write().get_fid() {
            Some(id) => id,
            None => {
                panic!("9pfs: No enough fids! Check fid_MAX constrant or fid leaky.");
            }
        };

        // get two new dfid for old dir and new dir.
        const ENOENT: u8 = 2;
        let result = if name.is_empty() {
            self.inner.write().twalk(*self.fid, fid, 0, &[])
        } else {
            self.inner.write().twalk(*self.fid, fid, 1, &[name])
        };

        match result {
            Ok(_) => {
                let node = CommonNode::new(
                    fid,
                    Some(self.this.clone()),
                    self.inner.clone(),
                    self.protocol.clone(),
                );
                match rest {
                    Some(rpath) => node.try_get(rpath),
                    None => Ok(node),
                }
            }
            // No such file or directory
            Err(ENOENT) => {
                self.inner.write().recycle_fid(fid);
                debug!("try_get failed {:?}=={}+{:?}", path, name, rest);
                Err(VfsError::NotFound)
            }
            Err(ecode) => {
                self.inner.write().recycle_fid(fid);
                error!("Failed when getting node in 9pfs, ecode:{}", ecode);
                Err(VfsError::BadState)
            }
        }
    }

    fn get_in_9pfs(&self, path: &str) -> VfsResult<Arc<CommonNode>> {
        let splited: Vec<&str> = path
            .split('/')
            .filter(|&x| !x.is_empty() && (x != "."))
            .collect();

        // get two new dfid for old dir and new dir.
        let new_fid = match self.inner.write().get_fid() {
            Some(id) => id,
            None => {
                panic!("9pfs: No enough fids! Check fid_MAX constrant or fid leaky.");
            }
        };

        // Operations in 9pfs
        let result = self
            .inner
            .write()
            .twalk(*self.fid, new_fid, splited.len() as u16, &splited);

        match result {
            Ok(_) => Ok(CommonNode::new(
                new_fid,
                Some(self.this.clone()),
                self.inner.clone(),
                self.protocol.clone(),
            )),
            Err(_) => Err(VfsError::BadState),
        }
    }
}

impl Drop for CommonNode {
    fn drop(&mut self) {
        // pay attention to AA-deadlock
        let result = self.inner.write().tclunk(*self.fid);
        const ENOENT: u8 = 2;
        match result {
            Ok(_) | Err(ENOENT) => {
                self.inner.write().recycle_fid(*self.fid);
            }
            Err(_) => {
                error!(
                    "9pfs(fid={}) drop failed! It may cause fid leaky problem. ",
                    *self.fid
                )
            }
        }
    }
}

impl VfsNodeOps for CommonNode {
    /// Renames or moves existing file or directory.
    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
        let (src_prefixs, old_name) = if let Some(src_sindex) = src_path.rfind('/') {
            (&src_path[..src_sindex], &src_path[src_sindex + 1..])
        } else {
            ("", src_path)
        };

        let (dst_prefixs, new_name) = if let Some(dst_sindex) = src_path.rfind('/') {
            (&dst_path[..dst_sindex], &dst_path[dst_sindex + 1..])
        } else {
            ("", dst_path)
        };

        debug!(
            "9pfs src_path:{} dst_path:{}, src_prefixs:{:?}, dst_prefixs:{:?}",
            src_path, dst_path, src_prefixs, dst_prefixs
        );

        let src_result = self.get_in_9pfs(src_prefixs);
        let dst_result = self.get_in_9pfs(dst_prefixs);

        if let (Ok(src_dnode), Ok(dst_dnode)) = (src_result, dst_result) {
            let src_fid = *src_dnode.fid;
            let dst_fid = *dst_dnode.fid;
            handle_result!(
                self.inner
                    .write()
                    .trename_at(src_fid, old_name, dst_fid, new_name),
                "9pfs rename_at failed! error code: {}"
            );
        } else {
            //create a new file and write content from original file.
            let src_fnode = self.try_get(src_path)?;
            let _ = self.create(dst_path, src_fnode.get_attr()?.file_type());
            let dst_fnode = self.try_get(dst_path)?;

            let mut buffer = [0_u8; 1024]; // a length for one turn to read and write
            let mut offset = 0;
            loop {
                let length = src_fnode.read_at(offset, &mut buffer)?;
                if length == 0 {
                    break;
                }
                dst_fnode.write_at(offset, &buffer[..length])?;
                offset += length as u64;
            }
            src_fnode.remove("")?;
        }
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        if *self.protocol == "9P2000.L" {
            let resp = self.inner.write().tgetattr(*self.fid, 0x3fff_u64);
            debug!("get_attr {:?}", resp);
            match resp {
                Ok(stat) if stat.get_ftype() == 0o4 => {
                    let mut attr = VfsNodeAttr::new_dir(stat.get_size(), stat.get_blk_num());
                    let mode = stat.get_perm() as u16 & 0o777_u16;
                    attr.set_perm(VfsNodePerm::from_bits(mode).unwrap());
                    Ok(attr)
                }
                Ok(stat) if stat.get_ftype() == 0o10 => {
                    let mut attr = VfsNodeAttr::new_file(stat.get_size(), stat.get_blk_num());
                    let mode = stat.get_perm() as u16 & 0o777_u16;
                    attr.set_perm(VfsNodePerm::from_bits(mode).unwrap());
                    Ok(attr)
                }
                _ => Err(VfsError::BadState),
            }
        } else if *self.protocol == "9P2000.u" {
            let resp = self.inner.write().tstat(*self.fid);
            match resp {
                Ok(stat) if stat.get_ftype() == 0o4 => {
                    let mut attr = VfsNodeAttr::new_dir(stat.get_length(), stat.get_blk_num());
                    let mode = stat.get_perm() as u16 & 0o777_u16;
                    attr.set_perm(VfsNodePerm::from_bits(mode).unwrap());
                    Ok(attr)
                }
                Ok(stat) if stat.get_ftype() == 0o10 => {
                    let mut attr = VfsNodeAttr::new_file(stat.get_length(), stat.get_blk_num());
                    let mode = stat.get_perm() as u16 & 0o777_u16;
                    attr.set_perm(VfsNodePerm::from_bits(mode).unwrap());
                    Ok(attr)
                }
                _ => Err(VfsError::BadState),
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
        debug!("lookup 9pfs: {}", path);
        self.try_get(path)
    }

    fn read_dir(&self, start_idx: usize, vfs_dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        debug!("9pfs reading dirents: start_idx = {:x?}", start_idx);
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

        let mut item_iter = dirents
            .iter()
            .filter(|&e| !(e.get_name().eq(".") || e.get_name().eq("..")))
            .skip(start_idx.max(2) - 2); // read from start_idx
        for (i, ent) in vfs_dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some(entry) = item_iter.next() {
                        let file_type = match entry.get_type() {
                            0o1_u8 => VfsNodeType::Fifo,
                            0o2_u8 => VfsNodeType::CharDevice,
                            0o4_u8 => VfsNodeType::Dir,
                            0o6_u8 => VfsNodeType::BlockDevice,
                            0o10_u8 => VfsNodeType::File,
                            0o12_u8 => VfsNodeType::SymLink,
                            0o14_u8 => VfsNodeType::Socket,
                            _ => panic!("9pfs: Unexpected file type found!"),
                        };
                        *ent = VfsDirEntry::new(entry.get_name(), file_type);
                    } else {
                        debug!("9pfs read dirents finished: start_idx = {:x?}", start_idx);
                        return Ok(i);
                    }
                }
            }
        }
        debug!("9pfs read dirents finished: start_idx = {:x?}", start_idx);
        Ok(vfs_dirents.len())
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at 9pfs: {}", ty, path);

        let (name, rest) = split_path(path);
        if let Some(rpath) = rest {
            self.try_get(name)?.create(rpath, ty)
        } else {
            self.create_node(name, ty)
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove at 9pfs: {}", path);
        match split_path(path) {
            ("", None) | (".", None) => match self.inner.write().tremove(*self.fid) {
                Ok(_) => Ok(()),
                Err(_) => Err(VfsError::BadState),
            },
            _ => self.try_get(path)?.remove(""),
        }
    }

    // Operation only for file usually
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
        debug!("read 9pid:{} length: {}", self.fid, buf.len());
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
        debug!("write 9pid:{} length: {}", self.fid, buf.len());
        let mut dev = self.inner.write();
        let mut write_len = buf.len();
        let mut offset_ptr = 0;
        while write_len > 0 {
            let target_buf = &buf[offset_ptr..];
            let wlen = match dev.twrite(*self.fid, offset + offset_ptr as u64, target_buf) {
                Ok(writed_length) => writed_length,
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

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}
