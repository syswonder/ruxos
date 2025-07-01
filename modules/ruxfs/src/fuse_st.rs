/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! fuse protocol definitions

#![allow(dead_code)]

use alloc::fmt::Debug;
use alloc::fmt::Error;
use alloc::fmt::Formatter;
use alloc::string::String;
use axfs_vfs::VfsNodeType;

/// FUSE operation codes.
#[derive(Debug, Clone, Copy)]
pub enum FuseOpcode {
    /// lookup a file by name
    FuseLookup = 1,
    /// forget a file, no reply
    FuseForget = 2,
    /// get file attributes
    FuseGetattr = 3,
    /// set file attributes
    FuseSetattr = 4,
    /// read a symbolic link
    FuseReadlink = 5,
    /// create a symbolic link
    FuseSymlink = 6,
    /// create a file
    FuseMknod = 8,
    /// create a directory
    FuseMkdir = 9,
    /// remove a file
    FuseUnlink = 10,
    /// remove a directory
    FuseRmdir = 11,
    /// rename a file or directory
    FuseRename = 12,
    /// create a hard link
    FuseLink = 13,
    /// open a file
    FuseOpen = 14,
    /// read a file
    FuseRead = 15,
    /// write a file
    FuseWrite = 16,
    /// get file system statistics
    FuseStatfs = 17,
    /// release a file
    FuseRelease = 18,
    /// release a file with flags
    FuseFsync = 20,
    /// set extended attributes
    FuseSetxattr = 21,
    /// get extended attributes
    FuseGetxattr = 22,
    /// list extended attributes
    FuseListxattr = 23,
    /// remove an extended attribute
    FuseRemovexattr = 24,
    /// flush a file
    FuseFlush = 25,
    /// initialize the FUSE file system
    FuseInit = 26,
    /// open a directory
    FuseOpendir = 27,
    /// read a directory
    FuseReaddir = 28,
    /// release a directory
    FuseReleasedir = 29,
    /// synchronize a directory
    FuseFsyncdir = 30,
    /// get or set file locks
    FuseGetlk = 31,
    /// set file locks
    FuseSetlk = 32,
    /// set file locks with wait
    FuseSetlkw = 33,
    /// access a file
    FuseAccess = 34,
    /// create a file with specific flags
    FuseCreate = 35,
    /// handle an interrupt
    FuseInterrupt = 36,
    /// handle a file system notification
    FuseBmap = 37,
    /// destroy the FUSE file system
    FuseDestroy = 38,
    /// perform an ioctl operation
    FuseIoctl = 39,
    /// poll for file system events
    FusePoll = 40,
    /// reply to a notification
    FuseNotifyReply = 41,
    /// batch forget files
    FuseBatchForget = 42,
    /// allocate space for a file
    FuseFallocate = 43,
    /// read directory with plus
    FuseReaddirplus = 44,
    /// rename a file or directory with flags
    FuseRename2 = 45,
    /// seek within a file
    FuseLseek = 46,
    /// copy a file range
    FuseCopyFileRange = 47,
    /// set up a mapping for a file
    FuseSetupmapping = 48,
    /// remove a mapping for a file
    FuseRemovemapping = 49,
    /// synchronize file system state
    FuseSyncfs = 50,
    /// create a temporary file
    FuseTmpfile = 51,
}

/// FUSE file open flags.
pub mod fuse_open_flags {
    /// Open a file with direct I/O.
    pub const FOPEN_DIRECT_IO: u32 = 1 << 0;
    /// Keep the file's cache.
    pub const FOPEN_KEEP_CACHE: u32 = 1 << 1;
    /// Open a file that is not seekable.
    pub const FOPEN_NONSEEKABLE: u32 = 1 << 2;
    /// Open a file in a cache directory.
    pub const FOPEN_CACHE_DIR: u32 = 1 << 3;
    /// Open a file as a stream.
    pub const FOPEN_STREAM: u32 = 1 << 4;
    /// Do not flush the file after writing.
    pub const FOPEN_NOFLUSH: u32 = 1 << 5;
    /// Allow parallel direct writes to the file.
    pub const FOPEN_PARALLEL_DIRECT_WRITES: u32 = 1 << 6;
}

/// FUSE file attribute bitmasks for fuse_setattr_in.valid
pub mod fuse_setattr_bitmasks {
    /// Bitmask for file mode
    pub const FATTR_MODE: u32 = 1 << 0;
    /// Bitmask for user ID
    pub const FATTR_UID: u32 = 1 << 1;
    /// Bitmask for group ID
    pub const FATTR_GID: u32 = 1 << 2;
    /// Bitmask for file size
    pub const FATTR_SIZE: u32 = 1 << 3;
    /// Bitmask for access time
    pub const FATTR_ATIME: u32 = 1 << 4;
    /// Bitmask for modification time
    pub const FATTR_MTIME: u32 = 1 << 5;
    /// Bitmask for file handle
    pub const FATTR_FH: u32 = 1 << 6;
    /// Bitmask for file attributes
    pub const FATTR_ATIME_NOW: u32 = 1 << 7;
    /// Bitmask for modification time
    pub const FATTR_MTIME_NOW: u32 = 1 << 8;
    /// Bitmask for lock owner
    pub const FATTR_LOCKOWNER: u32 = 1 << 9;
    /// Bitmask for change time
    pub const FATTR_CTIME: u32 = 1 << 10;
    /// Bitmask for killing setuid/setgid bits
    pub const FATTR_KILL_SUIDGID: u32 = 1 << 11;
}

/// FUSE file system initialization flags
pub mod fuse_init_flags {
    /// FUSE file system supports asynchronous reads
    pub const FUSE_ASYNC_READ: u32 = 1 << 0;
    /// FUSE file system supports POSIX locks
    pub const FUSE_POSIX_LOCKS: u32 = 1 << 1;
    /// FUSE file system supports file operations
    pub const FUSE_FILE_OPS: u32 = 1 << 2;
    /// FUSE file system supports atomic open with truncation
    pub const FUSE_ATOMIC_O_TRUNC: u32 = 1 << 3;
    /// FUSE file system supports export
    pub const FUSE_EXPORT_SUPPORT: u32 = 1 << 4;
    /// FUSE file system supports big writes
    pub const FUSE_BIG_WRITES: u32 = 1 << 5;
    /// FUSE file system does not mask file operations
    pub const FUSE_DONT_MASK: u32 = 1 << 6;
    /// FUSE file system supports splice write operations
    pub const FUSE_SPLICE_WRITE: u32 = 1 << 7;
    /// FUSE file system supports splice move operations
    pub const FUSE_SPLICE_MOVE: u32 = 1 << 8;
    /// FUSE file system supports splice read operations
    pub const FUSE_SPLICE_READ: u32 = 1 << 9;
    /// FUSE file system supports flock locks
    pub const FUSE_FLOCK_LOCKS: u32 = 1 << 10;
    /// FUSE file system supports ioctl operations in the directory
    pub const FUSE_HAS_IOCTL_DIR: u32 = 1 << 11;
    /// FUSE file system automatically invalidates data
    pub const FUSE_AUTO_INVAL_DATA: u32 = 1 << 12;
    /// FUSE file system supports readdirplus operations
    pub const FUSE_DO_READDIRPLUS: u32 = 1 << 13;
    /// FUSE file system supports readdirplus operations automatically
    pub const FUSE_READDIRPLUS_AUTO: u32 = 1 << 14;
    /// FUSE file system supports asynchronous direct I/O
    pub const FUSE_ASYNC_DIO: u32 = 1 << 15;
    /// FUSE file system supports writeback cache
    pub const FUSE_WRITEBACK_CACHE: u32 = 1 << 16;
    /// FUSE file system does not support open operations
    pub const FUSE_NO_OPEN_SUPPORT: u32 = 1 << 17;
    /// FUSE file system supports parallel directory operations
    pub const FUSE_PARALLEL_DIROPS: u32 = 1 << 18;
    /// FUSE file system supports killpriv operations
    pub const FUSE_HANDLE_KILLPRIV: u32 = 1 << 19;
    /// FUSE file system supports POSIX ACLs
    pub const FUSE_POSIX_ACL: u32 = 1 << 20;
    /// FUSE file system supports abort operations
    pub const FUSE_ABORT_ERROR: u32 = 1 << 21;
    /// FUSE file system supports a maximum of 4GB pages
    pub const FUSE_MAX_PAGES: u32 = 1 << 22;
    /// FUSE file system supports symlink caching
    pub const FUSE_CACHE_SYMLINKS: u32 = 1 << 23;
    /// FUSE file system does not support opendir operations
    pub const FUSE_NO_OPENDIR_SUPPORT: u32 = 1 << 24;
    /// FUSE file system explicitly invalidates data
    pub const FUSE_EXPLICIT_INVAL_DATA: u32 = 1 << 25;
    /// FUSE file system supports file handle caching
    pub const FUSE_MAP_ALIGNMENT: u32 = 1 << 26;
    /// FUSE file system supports submounts
    pub const FUSE_SUBMOUNTS: u32 = 1 << 27;
    /// FUSE file system supports killpriv operations in version 2
    pub const FUSE_HANDLE_KILLPRIV_V2: u32 = 1 << 28;
    /// FUSE file system supports extended attributes
    pub const FUSE_SETXATTR_EXT: u32 = 1 << 29;
    /// FUSE file system supports extended initialization
    pub const FUSE_INIT_EXT: u32 = 1 << 30;
    /// FUSE file system supports reserved flags
    pub const FUSE_INIT_RESERVED: u32 = 1 << 31;
    /// FUSE file system supports file system security context
    pub const FUSE_SECURITY_CTX: u64 = 1 << 32;
    /// FUSE file system supports DAX (Direct Access) inodes
    pub const FUSE_HAS_INODE_DAX: u64 = 1 << 33;
    /// FUSE file system supports group creation
    pub const FUSE_CREATE_SUPP_GROUP: u64 = 1 << 34;
}

/// FUSE file system flags
pub mod release_flags {
    /// FUSE release flags for flushing data
    pub const FUSE_RELEASE_FLUSH: u32 = 1 << 0;
    /// FUSE release flags for unlocking file locks
    pub const FUSE_RELEASE_FLOCK_UNLOCK: u32 = 1 << 1;
}

/// FUSE getattr flags
pub mod getattr_flags {
    /// FUSE getattr flags for file handle
    pub const FUSE_GETATTR_FH: u32 = 1 << 0;
}

/// FUSE write flags
pub mod write_flags {
    /// FUSE write flags for caching
    pub const FUSE_WRITE_CACHE: u32 = 1 << 0;
    /// FUSE write flags for locking
    pub const FUSE_WRITE_LOCKOWNER: u32 = 1 << 1;
    /// FUSE write flags for killing setuid/setgid bits
    pub const FUSE_WRITE_KILL_SUIDGID: u32 = 1 << 2;
}

/// FUSE read flags
pub mod read_flags {
    /// FUSE read flags for direct I/O
    pub const FUSE_READ_LOCKOWNER: u32 = 1 << 1;
}

/// FUSE request header
#[derive(Debug, Clone, Copy)]
pub struct FuseInHeader {
    // 40 bytes
    len: u32,     // length of the request = sizeof(fuse_in_header) = 32
    opcode: u32,  // eg. FUSE_GETATTR = 3
    unique: u64,  // unique request ID
    nodeid: u64,  // inode number
    uid: u32,     // user ID
    gid: u32,     // group ID
    pid: u32,     // process ID
    padding: u32, // padding
}

impl FuseInHeader {
    /// Create a new FuseInHeader.
    pub fn new(
        len: u32,
        opcode: u32,
        unique: u64,
        nodeid: u64,
        uid: u32,
        gid: u32,
        pid: u32,
    ) -> Self {
        Self {
            len,
            opcode,
            unique,
            nodeid,
            uid,
            gid,
            pid,
            padding: 0,
        }
    }

    /// Set the length of the request.
    pub fn set_len(&mut self, len: u32) {
        self.len = len;
    }

    /// Set the opcode of the request
    pub fn set_opcode(&mut self, opcode: u32) {
        self.opcode = opcode;
    }

    /// Set the unique request ID
    pub fn set_unique(&mut self, unique: u64) {
        self.unique = unique;
    }

    /// Set the inode number
    pub fn set_nodeid(&mut self, nodeid: u64) {
        self.nodeid = nodeid;
    }

    /// Set the user ID
    pub fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    /// Set the group ID
    pub fn set_gid(&mut self, gid: u32) {
        self.gid = gid;
    }

    /// Set the process ID
    pub fn set_pid(&mut self, pid: u32) {
        self.pid = pid;
    }

    /// Print the FuseInHeader
    pub fn print(&self) {
        debug!("FuseInHeader: len: {:?}, opcode: {:?}, unique: {:?}, nodeid: {:?}, uid: {:?}, gid: {:?}, pid: {:?}, padding: {:?}", self.len, self.opcode, self.unique, self.nodeid, self.uid, self.gid, self.pid, self.padding);
    }

    /// Write the FuseInHeader to a buffer
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.len.to_le_bytes());
        buf[4..8].copy_from_slice(&self.opcode.to_le_bytes());
        buf[8..16].copy_from_slice(&self.unique.to_le_bytes());
        buf[16..24].copy_from_slice(&self.nodeid.to_le_bytes());
        buf[24..28].copy_from_slice(&self.uid.to_le_bytes());
        buf[28..32].copy_from_slice(&self.gid.to_le_bytes());
        buf[32..36].copy_from_slice(&self.pid.to_le_bytes());
        buf[36..40].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE response header
#[derive(Debug, Clone, Copy)]
pub struct FuseOutHeader {
    // 16 bytes
    len: u32,    // length of the response
    error: i32,  // error code
    unique: u64, // unique request ID
}

impl FuseOutHeader {
    /// Create a new FuseOutHeader.
    pub fn new(len: u32, error: i32, unique: u64) -> Self {
        Self { len, error, unique }
    }

    /// Read a FuseOutHeader from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            len: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            error: i32::from_le_bytes(buf[4..8].try_into().unwrap()),
            unique: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
        }
    }

    /// Check if the response is successful (error code is 0).
    pub fn is_ok(&self) -> bool {
        self.error == 0
    }

    /// Get the length of the response.
    pub fn get_len(&self) -> u32 {
        self.len
    }

    /// Get the error code of the response.
    pub fn error(&self) -> i32 {
        self.error
    }

    /// Get the unique request ID of the response.
    pub fn get_unique(&self) -> u64 {
        self.unique
    }

    /// Set the length of the response.
    pub fn print(&self) {
        debug!(
            "fuse_out_header: len: {:?}, error: {:?}, unique: {:?}",
            self.len, self.error, self.unique
        );
    }

    /// Write the FuseOutHeader to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.len.to_le_bytes());
        buf[4..8].copy_from_slice(&self.error.to_le_bytes());
        buf[8..16].copy_from_slice(&self.unique.to_le_bytes());
    }
}

/// FUSE initialization request input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseInitIn {
    // 64 bytes
    major: u32,
    minor: u32,
    max_readahead: u32,
    flags: u32,
    flags2: u32,
    unused: [u32; 11],
}

impl FuseInitIn {
    /// Create a new FuseInitIn structure.
    pub fn new(
        major: u32,
        minor: u32,
        max_readahead: u32,
        flags: u32,
        flags2: u32,
        unused: [u32; 11],
    ) -> Self {
        Self {
            major,
            minor,
            max_readahead,
            flags,
            flags2,
            unused,
        }
    }

    /// Print the FuseInitIn structure.
    pub fn print(&self) {
        debug!("FuseInitIn: major: {:?}, minor: {:?}, max_readahead: {:#x}, flags: {:#x}, flags2: {:?}, unused: {:?}", self.major, self.minor, self.max_readahead, self.flags, self.flags2, self.unused);
    }

    /// Write the FuseInitIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.major.to_le_bytes());
        buf[4..8].copy_from_slice(&self.minor.to_le_bytes());
        buf[8..12].copy_from_slice(&self.max_readahead.to_le_bytes());
        buf[12..16].copy_from_slice(&self.flags.to_le_bytes());
        buf[16..20].copy_from_slice(&self.flags2.to_le_bytes());
        for (i, &val) in self.unused.iter().enumerate() {
            buf[20 + i * 4..24 + i * 4].copy_from_slice(&val.to_le_bytes());
        }
    }
}

/// FUSE initialization response output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseInitOut {
    // 64 bytes
    major: u32,
    minor: u32,
    max_readahead: u32,
    flags: u32,
    max_background: u16,
    congestion_threshold: u16,
    max_write: u32,
    time_gran: u32,
    max_pages: u16,
    map_alignment: u16,
    flags2: u32,
    unused: [u32; 7],
}

impl FuseInitOut {
    /// Create a new FuseInitOut structure.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        major: u32,
        minor: u32,
        max_readahead: u32,
        flags: u32,
        max_background: u16,
        congestion_threshold: u16,
        max_write: u32,
        time_gran: u32,
        max_pages: u16,
        map_alignment: u16,
        flags2: u32,
        unused: [u32; 7],
    ) -> Self {
        Self {
            major,
            minor,
            max_readahead,
            flags,
            max_background,
            congestion_threshold,
            max_write,
            time_gran,
            max_pages,
            map_alignment,
            flags2,
            unused,
        }
    }

    /// Read a FuseInitOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        trace!("fuseinitout from len: {:?}, buf: {:?}", buf.len(), buf);
        Self {
            major: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            minor: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            max_readahead: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            flags: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            max_background: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            congestion_threshold: u16::from_le_bytes(buf[18..20].try_into().unwrap()),
            max_write: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            time_gran: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            max_pages: u16::from_le_bytes(buf[28..30].try_into().unwrap()),
            map_alignment: u16::from_le_bytes(buf[30..32].try_into().unwrap()),
            flags2: u32::from_le_bytes(buf[32..36].try_into().unwrap()),
            unused: [
                u32::from_le_bytes(buf[36..40].try_into().unwrap()),
                u32::from_le_bytes(buf[40..44].try_into().unwrap()),
                u32::from_le_bytes(buf[44..48].try_into().unwrap()),
                u32::from_le_bytes(buf[48..52].try_into().unwrap()),
                u32::from_le_bytes(buf[52..56].try_into().unwrap()),
                u32::from_le_bytes(buf[56..60].try_into().unwrap()),
                u32::from_le_bytes(buf[60..64].try_into().unwrap()),
            ],
        }
    }

    /// Get the major version of the FUSE protocol.
    pub fn get_major(&self) -> u32 {
        self.major
    }

    /// Get the minor version of the FUSE protocol.
    pub fn get_minor(&self) -> u32 {
        self.minor
    }

    /// Get the maximum readahead size.
    pub fn get_max_readahead(&self) -> u32 {
        self.max_readahead
    }

    /// Get the flags of the FUSE file system.
    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    /// Get the maximum number of background requests.
    pub fn get_max_background(&self) -> u16 {
        self.max_background
    }

    /// Get the congestion threshold.
    pub fn get_congestion_threshold(&self) -> u16 {
        self.congestion_threshold
    }

    /// Get the maximum write size.
    pub fn get_max_write(&self) -> u32 {
        self.max_write
    }

    /// Get the time granularity.
    pub fn get_time_gran(&self) -> u32 {
        self.time_gran
    }

    /// Get the maximum number of pages.
    pub fn get_max_pages(&self) -> u16 {
        self.max_pages
    }

    /// Get the map alignment.
    pub fn get_map_alignment(&self) -> u16 {
        self.map_alignment
    }

    /// Get the flags2 of the FUSE file system.
    pub fn get_flags2(&self) -> u32 {
        self.flags2
    }

    /// Print the FuseInitOut structure.
    pub fn print(&self) {
        debug!("FuseInitOut: major: {:?}, minor: {:?}, max_readahead: {:#x}, flags: {:#x}, max_background: {:?}, congestion_threshold: {:?}, max_write: {:#x}, time_gran: {:?}, max_pages: {:?}, map_alignment: {:?}, flags2: {:?}, unused: {:?}", self.major, self.minor, self.max_readahead, self.flags, self.max_background, self.congestion_threshold, self.max_write, self.time_gran, self.max_pages, self.map_alignment, self.flags2, self.unused);
    }
}

/// FUSE getattr input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseGetattrIn {
    // 16 bytes
    getattr_flags: u32,
    dummy: u32,
    fh: u64,
}

impl FuseGetattrIn {
    /// Create a new FuseGetattrIn structure.
    pub fn new(getattr_flags: u32, dummy: u32, fh: u64) -> Self {
        Self {
            getattr_flags,
            dummy,
            fh,
        }
    }

    /// Print the FuseGetattrIn structure.
    pub fn print(&self) {
        debug!(
            "FuseGetattrIn: getattr_flags: {:#x}, dummy: {:?}, fh: {:#x}",
            self.getattr_flags, self.dummy, self.fh
        );
    }

    /// Write the FuseGetattrIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.getattr_flags.to_le_bytes());
        buf[4..8].copy_from_slice(&self.dummy.to_le_bytes());
        buf[8..16].copy_from_slice(&self.fh.to_le_bytes());
    }
}

/// FUSE file attributes structure
#[derive(Clone, Copy, Default)]
pub struct FuseAttr {
    // 88 bytes
    ino: u64,
    size: u64,
    blocks: u64,
    atime: u64,
    mtime: u64,
    ctime: u64,
    atimensec: u32,
    mtimensec: u32,
    ctimensec: u32,
    mode: u32,
    nlink: u32,
    uid: u32,
    gid: u32,
    rdev: u32,
    blksize: u32,
    flags: u32,
}

impl FuseAttr {
    /// Create a new FuseAttr structure.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ino: u64,
        size: u64,
        blocks: u64,
        atime: u64,
        mtime: u64,
        ctime: u64,
        atimensec: u32,
        mtimensec: u32,
        ctimensec: u32,
        mode: u32,
        nlink: u32,
        uid: u32,
        gid: u32,
        rdev: u32,
        blksize: u32,
        flags: u32,
    ) -> Self {
        Self {
            ino,
            size,
            blocks,
            atime,
            mtime,
            ctime,
            atimensec,
            mtimensec,
            ctimensec,
            mode,
            nlink,
            uid,
            gid,
            rdev,
            blksize,
            flags,
        }
    }

    /// Read a FuseAttr structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            ino: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            size: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            blocks: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            atime: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            mtime: u64::from_le_bytes(buf[32..40].try_into().unwrap()),
            ctime: u64::from_le_bytes(buf[40..48].try_into().unwrap()),
            atimensec: u32::from_le_bytes(buf[48..52].try_into().unwrap()),
            mtimensec: u32::from_le_bytes(buf[52..56].try_into().unwrap()),
            ctimensec: u32::from_le_bytes(buf[56..60].try_into().unwrap()),
            mode: u32::from_le_bytes(buf[60..64].try_into().unwrap()),
            nlink: u32::from_le_bytes(buf[64..68].try_into().unwrap()),
            uid: u32::from_le_bytes(buf[68..72].try_into().unwrap()),
            gid: u32::from_le_bytes(buf[72..76].try_into().unwrap()),
            rdev: u32::from_le_bytes(buf[76..80].try_into().unwrap()),
            blksize: u32::from_le_bytes(buf[80..84].try_into().unwrap()),
            flags: u32::from_le_bytes(buf[84..88].try_into().unwrap()),
        }
    }

    /// Get the size of the file.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Get the mode of the file.
    pub fn get_mode(&self) -> u32 {
        self.mode
    }

    /// Get the user ID of the file.
    pub fn get_uid(&self) -> u32 {
        self.uid
    }

    /// Get the group ID of the file.
    pub fn get_gid(&self) -> u32 {
        self.gid
    }

    /// Get the number of hard links to the file.
    pub fn get_nlink(&self) -> u32 {
        self.nlink
    }

    /// Get the inode number of the file.
    pub fn get_ino(&self) -> u64 {
        self.ino
    }

    /// Get the number of blocks allocated for the file.
    pub fn get_blocks(&self) -> u64 {
        self.blocks
    }

    /// Get the last access time of the file.
    pub fn get_atime(&self) -> u64 {
        self.atime
    }

    /// Get the last modification time of the file.
    pub fn get_mtime(&self) -> u64 {
        self.mtime
    }

    /// Get the last change time of the file.
    pub fn get_ctime(&self) -> u64 {
        self.ctime
    }

    /// Get the nanoseconds of the last access time.
    pub fn get_atimensec(&self) -> u32 {
        self.atimensec
    }

    /// Get the nanoseconds of the last modification time.
    pub fn get_mtimensec(&self) -> u32 {
        self.mtimensec
    }

    /// Get the nanoseconds of the last change time.
    pub fn get_ctimensec(&self) -> u32 {
        self.ctimensec
    }

    /// Get the device ID of the file.
    pub fn get_rdev(&self) -> u32 {
        self.rdev
    }

    /// Get the block size of the file.
    pub fn get_blksize(&self) -> u32 {
        self.blksize
    }

    /// Get the flags of the file.
    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    /// Get the node type of the file.
    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    /// Set the mode of the file.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Set the user ID of the file.
    pub fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    /// Set the group ID of the file.
    pub fn set_gid(&mut self, gid: u32) {
        self.gid = gid;
    }

    /// Set the number of hard links to the file.
    pub fn set_nlink(&mut self, nlink: u32) {
        self.nlink = nlink;
    }

    /// Set the inode number of the file.
    pub fn set_ino(&mut self, ino: u64) {
        self.ino = ino;
    }

    /// Set the number of blocks allocated for the file.
    pub fn set_blocks(&mut self, blocks: u64) {
        self.blocks = blocks;
    }

    /// Set the last access time of the file.
    pub fn set_atime(&mut self, atime: u64) {
        self.atime = atime;
    }

    /// Set the last modification time of the file.
    pub fn set_mtime(&mut self, mtime: u64) {
        self.mtime = mtime;
    }

    /// Set the last change time of the file.
    pub fn set_ctime(&mut self, ctime: u64) {
        self.ctime = ctime;
    }

    /// Set the nanoseconds of the last access time.
    pub fn set_atimensec(&mut self, atimensec: u32) {
        self.atimensec = atimensec;
    }

    /// Set the nanoseconds of the last modification time.
    pub fn set_mtimensec(&mut self, mtimensec: u32) {
        self.mtimensec = mtimensec;
    }

    /// Set the nanoseconds of the last change time.
    pub fn set_ctimensec(&mut self, ctimensec: u32) {
        self.ctimensec = ctimensec;
    }

    /// Set the device ID of the file.
    pub fn set_rdev(&mut self, rdev: u32) {
        self.rdev = rdev;
    }

    /// Set the block size of the file.
    pub fn set_blksize(&mut self, blksize: u32) {
        self.blksize = blksize;
    }

    /// Set the flags of the file.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Print the FuseAttr structure.
    pub fn print(&self) {
        debug!("FuseAttr: ino: {:?}, size: {:?}, blocks: {:?}, atime: {:?}, mtime: {:?}, ctime: {:?}, atimensec: {:?}, mtimensec: {:?}, ctimensec: {:?}, mode: {:#x}, nlink: {:?}, uid: {:?}, gid: {:?}, rdev: {:#x}, blksize: {:?}, flags: {:#x}", self.ino, self.size, self.blocks, self.atime, self.mtime, self.ctime, self.atimensec, self.mtimensec, self.ctimensec, self.mode, self.nlink, self.uid, self.gid, self.rdev, self.blksize, self.flags);
    }

    /// Write the FuseAttr structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.ino.to_le_bytes());
        buf[8..16].copy_from_slice(&self.size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.blocks.to_le_bytes());
        buf[24..32].copy_from_slice(&self.atime.to_le_bytes());
        buf[32..40].copy_from_slice(&self.mtime.to_le_bytes());
        buf[40..48].copy_from_slice(&self.ctime.to_le_bytes());
        buf[48..52].copy_from_slice(&self.atimensec.to_le_bytes());
        buf[52..56].copy_from_slice(&self.mtimensec.to_le_bytes());
        buf[56..60].copy_from_slice(&self.ctimensec.to_le_bytes());
        buf[60..64].copy_from_slice(&self.mode.to_le_bytes());
        buf[64..68].copy_from_slice(&self.nlink.to_le_bytes());
        buf[68..72].copy_from_slice(&self.uid.to_le_bytes());
        buf[72..76].copy_from_slice(&self.gid.to_le_bytes());
        buf[76..80].copy_from_slice(&self.rdev.to_le_bytes());
        buf[80..84].copy_from_slice(&self.blksize.to_le_bytes());
        buf[84..88].copy_from_slice(&self.flags.to_le_bytes());
    }
}

impl Debug for FuseAttr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "FuseAttr: {{ ino: {:?}, size: {:?}, blocks: {:?}, atime: {:?}, mtime: {:?}, ctime: {:?}, atimensec: {:?}, mtimensec: {:?}, ctimensec: {:?}, mode: {:#x}, nlink: {:?}, uid: {:?}, gid: {:?}, rdev: {:#x}, blksize: {:?}, flags: {:#x} }}", self.ino, self.size, self.blocks, self.atime, self.mtime, self.ctime, self.atimensec, self.mtimensec, self.ctimensec, self.mode, self.nlink, self.uid, self.gid, self.rdev, self.blksize, self.flags)
    }
}

/// FUSE file system statistics structure
#[derive(Debug, Clone, Copy, Default)]
pub struct FuseKstatfs {
    // 80 bytes
    blocks: u64,
    bfree: u64,
    bavail: u64,
    files: u64,
    ffree: u64,
    bsize: u32,
    namelen: u32,
    frsize: u32,
    padding: u32,
    spare: [u32; 6],
}

impl FuseKstatfs {
    /// Create a new FuseKstatfs structure.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        blocks: u64,
        bfree: u64,
        bavail: u64,
        files: u64,
        ffree: u64,
        bsize: u32,
        namelen: u32,
        frsize: u32,
        padding: u32,
        spare: [u32; 6],
    ) -> Self {
        Self {
            blocks,
            bfree,
            bavail,
            files,
            ffree,
            bsize,
            namelen,
            frsize,
            padding,
            spare,
        }
    }

    /// Read a FuseKstatfs structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            blocks: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            bfree: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            bavail: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            files: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            ffree: u64::from_le_bytes(buf[32..40].try_into().unwrap()),
            bsize: u32::from_le_bytes(buf[40..44].try_into().unwrap()),
            namelen: u32::from_le_bytes(buf[44..48].try_into().unwrap()),
            frsize: u32::from_le_bytes(buf[48..52].try_into().unwrap()),
            padding: u32::from_le_bytes(buf[52..56].try_into().unwrap()),
            spare: [
                u32::from_le_bytes(buf[56..60].try_into().unwrap()),
                u32::from_le_bytes(buf[60..64].try_into().unwrap()),
                u32::from_le_bytes(buf[64..68].try_into().unwrap()),
                u32::from_le_bytes(buf[68..72].try_into().unwrap()),
                u32::from_le_bytes(buf[72..76].try_into().unwrap()),
                u32::from_le_bytes(buf[76..80].try_into().unwrap()),
            ],
        }
    }

    /// Print the FuseKstatfs structure.
    pub fn print(&self) {
        debug!("FuseKstatfs: blocks: {:?}, bfree: {:?}, bavail: {:?}, files: {:?}, ffree: {:?}, bsize: {:?}, namelen: {:?}, frsize: {:?}, padding: {:?}, spare: {:?}", self.blocks, self.bfree, self.bavail, self.files, self.ffree, self.bsize, self.namelen, self.frsize, self.padding, self.spare);
    }
}

/// FUSE file lock structure
#[derive(Debug, Clone, Copy)]
pub struct FuseFileLock {
    // 24 bytes
    start: u64,
    end: u64,
    type_: u32,
    pid: u32,
}

impl FuseFileLock {
    /// Create a new FuseFileLock structure.
    pub fn new(start: u64, end: u64, type_: u32, pid: u32) -> Self {
        Self {
            start,
            end,
            type_,
            pid,
        }
    }

    /// Read a FuseFileLock structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            start: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            end: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            type_: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            pid: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
        }
    }

    /// Print the FuseFileLock structure.
    pub fn print(&self) {
        debug!(
            "FuseFileLock: start: {:?}, end: {:?}, type: {:?}, pid: {:?}",
            self.start, self.end, self.type_, self.pid
        );
    }
}

/// FUSE file attributes output structure
#[derive(Debug, Clone, Copy, Default)]
pub struct FuseAttrOut {
    // 104 bytes
    attr_valid: u64,
    attr_valid_nsec: u32,
    dummy: u32,
    attr: FuseAttr,
}

impl FuseAttrOut {
    /// Create a new FuseAttrOut structure.
    pub fn new(attr_valid: u64, attr_valid_nsec: u32, dummy: u32, attr: FuseAttr) -> Self {
        Self {
            attr_valid,
            attr_valid_nsec,
            dummy,
            attr,
        }
    }

    /// Read a FuseAttrOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            attr_valid: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            attr_valid_nsec: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            dummy: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            attr: FuseAttr::read_from(&buf[16..104]),
        }
    }

    /// Get the attribute valid time.
    pub fn get_attr_valid(&self) -> u64 {
        self.attr_valid
    }

    /// Get the attribute valid time in nanoseconds.
    pub fn get_attr_valid_nsec(&self) -> u32 {
        self.attr_valid_nsec
    }

    /// Get the dummy value.
    pub fn get_dummy(&self) -> u32 {
        self.dummy
    }

    /// Get the file attributes.
    pub fn get_attr(&self) -> FuseAttr {
        self.attr
    }

    /// Get the size of the file.
    pub fn get_size(&self) -> u64 {
        self.attr.size
    }

    /// Print the FuseAttrOut structure.
    pub fn print(&self) {
        debug!(
            "FuseAttrOut: attr_valid: {:?}, attr_valid_nsec: {:?}, dummy: {:?}, attr: {:?}",
            self.attr_valid, self.attr_valid_nsec, self.dummy, self.attr
        );
    }
}

/// FUSE entry output structure
#[derive(Debug, Clone, Copy, Default)]
pub struct FuseEntryOut {
    // 128 bytes
    nodeid: u64, /* Inode ID */
    generation: u64, /* Inode generation: nodeid:gen must
                 be unique for the fs's lifetime */
    entry_valid: u64, /* Cache timeout for the name */
    attr_valid: u64,  /* Cache timeout for the attributes */
    entry_valid_nsec: u32,
    attr_valid_nsec: u32,
    attr: FuseAttr,
}

impl FuseEntryOut {
    /// Create a new FuseEntryOut structure.
    pub fn new(
        nodeid: u64,
        generation: u64,
        entry_valid: u64,
        attr_valid: u64,
        entry_valid_nsec: u32,
        attr_valid_nsec: u32,
        attr: FuseAttr,
    ) -> Self {
        Self {
            nodeid,
            generation,
            entry_valid,
            attr_valid,
            entry_valid_nsec,
            attr_valid_nsec,
            attr,
        }
    }

    /// Read a FuseEntryOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            nodeid: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            generation: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            entry_valid: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            attr_valid: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            entry_valid_nsec: u32::from_le_bytes(buf[32..36].try_into().unwrap()),
            attr_valid_nsec: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
            attr: FuseAttr::read_from(&buf[40..128]),
        }
    }

    /// Get the node ID of the entry.
    pub fn get_nodeid(&self) -> u64 {
        self.nodeid
    }

    /// Get the generation of the entry.
    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    /// Get the entry valid time.
    pub fn get_entry_valid(&self) -> u64 {
        self.entry_valid
    }

    /// Get the attribute valid time.
    pub fn get_attr_valid(&self) -> u64 {
        self.attr_valid
    }

    /// Get the entry valid time in nanoseconds.
    pub fn get_entry_valid_nsec(&self) -> u32 {
        self.entry_valid_nsec
    }

    /// Get the attribute valid time in nanoseconds.
    pub fn get_attr_valid_nsec(&self) -> u32 {
        self.attr_valid_nsec
    }

    /// Get the file attributes.
    pub fn get_attr(&self) -> FuseAttr {
        self.attr
    }

    /// Get the number of hard links to the file.
    pub fn get_nlink(&self) -> u32 {
        self.attr.nlink
    }

    /// Get the size of the file.
    pub fn get_size(&self) -> u64 {
        self.attr.size
    }

    /// Print the FuseEntryOut structure.
    pub fn print(&self) {
        debug!("FuseEntryOut: nodeid: {:?}, generation: {:?}, entry_valid: {:?}, attr_valid: {:?}, entry_valid_nsec: {:?}, attr_valid_nsec: {:?}, attr: {:?}", self.nodeid, self.generation, self.entry_valid, self.attr_valid, self.entry_valid_nsec, self.attr_valid_nsec, self.attr);
    }
}

/// FUSE setattr input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseSetattrIn {
    // 88 bytes
    valid: u32,
    padding: u32,
    fh: u64,
    size: u64,
    lock_owner: u64,
    atime: u64,
    mtime: u64,
    ctime: u64,
    atimensec: u32,
    mtimensec: u32,
    ctimensec: u32,
    mode: u32,
    unused4: u32,
    uid: u32,
    gid: u32,
    unused5: u32,
}

impl FuseSetattrIn {
    /// Create a new FuseSetattrIn structure.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        valid: u32,
        padding: u32,
        fh: u64,
        size: u64,
        lock_owner: u64,
        atime: u64,
        mtime: u64,
        ctime: u64,
        atimensec: u32,
        mtimensec: u32,
        ctimensec: u32,
        mode: u32,
        unused4: u32,
        uid: u32,
        gid: u32,
        unused5: u32,
    ) -> Self {
        Self {
            valid,
            padding,
            fh,
            size,
            lock_owner,
            atime,
            mtime,
            ctime,
            atimensec,
            mtimensec,
            ctimensec,
            mode,
            unused4,
            uid,
            gid,
            unused5,
        }
    }

    /// Set the valid flags.
    pub fn set_valid(&mut self, valid: u32) {
        self.valid = valid;
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the size of the file.
    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    /// Set the lock owner.
    pub fn set_lock_owner(&mut self, lock_owner: u64) {
        self.lock_owner = lock_owner;
    }

    /// Set the access time.
    pub fn set_atime(&mut self, atime: u64) {
        self.atime = atime;
    }

    /// Set the modification time.
    pub fn set_mtime(&mut self, mtime: u64) {
        self.mtime = mtime;
    }

    /// Set the change time.
    pub fn set_ctime(&mut self, ctime: u64) {
        self.ctime = ctime;
    }

    /// Set the nanoseconds of the access time.
    pub fn set_atimensec(&mut self, atimensec: u32) {
        self.atimensec = atimensec;
    }

    /// Set the nanoseconds of the modification time.
    pub fn set_mtimensec(&mut self, mtimensec: u32) {
        self.mtimensec = mtimensec;
    }

    /// Set the nanoseconds of the change time.
    pub fn set_ctimensec(&mut self, ctimensec: u32) {
        self.ctimensec = ctimensec;
    }

    /// Set the mode of the file.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Set the user ID of the file.
    pub fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    /// Set the group ID of the file.
    pub fn set_gid(&mut self, gid: u32) {
        self.gid = gid;
    }

    /// Print the FuseSetattrIn structure.
    pub fn print(&self) {
        debug!("FuseSetattrIn: valid: {:?}, padding: {:?}, fh: {:#x}, size: {:?}, lock_owner: {:?}, atime: {:?}, mtime: {:?}, ctime: {:?}, atimensec: {:?}, mtimensec: {:?}, ctimensec: {:?}, mode: {:#x}, unused4: {:?}, uid: {:?}, gid: {:?}, unused5: {:?}", self.valid, self.padding, self.fh, self.size, self.lock_owner, self.atime, self.mtime, self.ctime, self.atimensec, self.mtimensec, self.ctimensec, self.mode, self.unused4, self.uid, self.gid, self.unused5);
    }

    /// Write the FuseSetattrIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.valid.to_le_bytes());
        buf[4..8].copy_from_slice(&self.padding.to_le_bytes());
        buf[8..16].copy_from_slice(&self.fh.to_le_bytes());
        buf[16..24].copy_from_slice(&self.size.to_le_bytes());
        buf[24..32].copy_from_slice(&self.lock_owner.to_le_bytes());
        buf[32..40].copy_from_slice(&self.atime.to_le_bytes());
        buf[40..48].copy_from_slice(&self.mtime.to_le_bytes());
        buf[48..56].copy_from_slice(&self.ctime.to_le_bytes());
        buf[56..60].copy_from_slice(&self.atimensec.to_le_bytes());
        buf[60..64].copy_from_slice(&self.mtimensec.to_le_bytes());
        buf[64..68].copy_from_slice(&self.ctimensec.to_le_bytes());
        buf[68..72].copy_from_slice(&self.mode.to_le_bytes());
        buf[72..76].copy_from_slice(&self.unused4.to_le_bytes());
        buf[76..80].copy_from_slice(&self.uid.to_le_bytes());
        buf[80..84].copy_from_slice(&self.gid.to_le_bytes());
        buf[84..88].copy_from_slice(&self.unused5.to_le_bytes());
    }
}

/// FUSE open input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseOpenIn {
    // 8 bytes
    flags: u32,
    open_flags: u32, /* FUSE_OPEN_... */
}

impl FuseOpenIn {
    /// Create a new FuseOpenIn structure.
    pub fn new(flags: u32, open_flags: u32) -> Self {
        Self { flags, open_flags }
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the open flags.
    pub fn set_open_flags(&mut self, open_flags: u32) {
        self.open_flags = open_flags;
    }

    /// Print the FuseOpenIn structure.
    pub fn print(&self) {
        debug!(
            "FuseOpenIn: flags: {:#x}, open_flags: {:#x}",
            self.flags, self.open_flags
        );
    }

    /// Write the FuseOpenIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.flags.to_le_bytes());
        buf[4..8].copy_from_slice(&self.open_flags.to_le_bytes());
    }
}

/// FUSE open output structure
#[derive(Debug, Clone, Copy, Default)]
pub struct FuseOpenOut {
    // 16 bytes
    fh: u64,
    open_flags: u32,
    padding: u32,
}

impl FuseOpenOut {
    /// Create a new FuseOpenOut structure.
    pub fn new(fh: u64, open_flags: u32) -> Self {
        Self {
            fh,
            open_flags,
            padding: 0,
        }
    }

    /// Read a FuseOpenOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            fh: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            open_flags: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            padding: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
        }
    }

    /// Get the file handle.
    pub fn get_fh(&self) -> u64 {
        self.fh
    }

    /// Get the open flags.
    pub fn get_open_flags(&self) -> u32 {
        self.open_flags
    }

    /// Print the FuseOpenOut structure.
    pub fn print(&self) {
        debug!(
            "FuseOpenOut: fh: {:#x}, open_flags: {:#x}, padding: {:?}",
            self.fh, self.open_flags, self.padding
        );
    }
}

/// FUSE read input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseReadIn {
    // 40 bytes
    fh: u64,
    offset: u64,
    size: u32,
    read_flags: u32,
    lock_owner: u64,
    flags: u32,
    padding: u32,
}

impl FuseReadIn {
    /// Create a new FuseReadIn structure.
    pub fn new(
        fh: u64,
        offset: u64,
        size: u32,
        read_flags: u32,
        lock_owner: u64,
        flags: u32,
    ) -> Self {
        Self {
            fh,
            offset,
            size,
            read_flags,
            lock_owner,
            flags,
            padding: 0,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the offset in the file.
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Set the size of the read operation.
    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    /// Set the read flags.
    pub fn set_read_flags(&mut self, read_flags: u32) {
        self.read_flags = read_flags;
    }

    /// Set the lock owner.
    pub fn set_lock_owner(&mut self, lock_owner: u64) {
        self.lock_owner = lock_owner;
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Print the FuseReadIn structure.
    pub fn print(&self) {
        debug!("FuseReadIn: fh: {:#x}, offset: {:?}, size: {:?}, read_flags: {:#x}, lock_owner: {:?}, flags: {:#x}, padding: {:?}", self.fh, self.offset, self.size, self.read_flags, self.lock_owner, self.flags, self.padding);
    }

    /// Write the FuseReadIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
        buf[16..20].copy_from_slice(&self.size.to_le_bytes());
        buf[20..24].copy_from_slice(&self.read_flags.to_le_bytes());
        buf[24..32].copy_from_slice(&self.lock_owner.to_le_bytes());
        buf[32..36].copy_from_slice(&self.flags.to_le_bytes());
    }
}

/// FUSE write input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseWriteIn {
    // 40 bytes
    fh: u64,
    offset: u64,
    size: u32,
    write_flags: u32,
    lock_owner: u64,
    flags: u32,
    padding: u32,
}

impl FuseWriteIn {
    /// Create a new FuseWriteIn structure.
    pub fn new(
        fh: u64,
        offset: u64,
        size: u32,
        write_flags: u32,
        lock_owner: u64,
        flags: u32,
    ) -> Self {
        Self {
            fh,
            offset,
            size,
            write_flags,
            lock_owner,
            flags,
            padding: 0,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the offset in the file.
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Set the size of the write operation.
    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    /// Set the write flags.
    pub fn set_write_flags(&mut self, write_flags: u32) {
        self.write_flags = write_flags;
    }

    /// Set the lock owner.
    pub fn set_lock_owner(&mut self, lock_owner: u64) {
        self.lock_owner = lock_owner;
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Print the FuseWriteIn structure.
    pub fn print(&self) {
        debug!("FuseWriteIn: fh: {:#x}, offset: {:?}, size: {:?}, write_flags: {:#x}, lock_owner: {:?}, flags: {:#x}, padding: {:?}", self.fh, self.offset, self.size, self.write_flags, self.lock_owner, self.flags, self.padding);
    }

    /// Write the FuseWriteIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
        buf[16..20].copy_from_slice(&self.size.to_le_bytes());
        buf[20..24].copy_from_slice(&self.write_flags.to_le_bytes());
        buf[24..32].copy_from_slice(&self.lock_owner.to_le_bytes());
        buf[32..36].copy_from_slice(&self.flags.to_le_bytes());
    }
}

/// FUSE write output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseWriteOut {
    // 8 bytes
    size: u32,
    padding: u32,
}

impl FuseWriteOut {
    /// Create a new FuseWriteOut structure.
    pub fn new(size: u32) -> Self {
        Self { size, padding: 0 }
    }

    /// Read a FuseWriteOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        }
    }

    /// Get the size of the written data.
    pub fn get_size(&self) -> u32 {
        self.size
    }

    /// Print the FuseWriteOut structure.
    pub fn print(&self) {
        debug!(
            "FuseWriteOut: size: {:?}, padding: {:?}",
            self.size, self.padding
        );
    }

    /// Write the FuseWriteOut structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.size.to_le_bytes());
    }
}

/// FUSE create input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseCreateIn {
    // 16 bytes
    flags: u32,
    mode: u32,
    umask: u32,
    open_flags: u32, /* FUSE_OPEN_... */
}

impl FuseCreateIn {
    /// Create a new FuseCreateIn structure.
    pub fn new(flags: u32, mode: u32, umask: u32, open_flags: u32) -> Self {
        Self {
            flags,
            mode,
            umask,
            open_flags,
        }
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the mode of the file.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Set the umask for the file.
    pub fn set_umask(&mut self, umask: u32) {
        self.umask = umask;
    }

    /// Set the open flags.
    pub fn set_open_flags(&mut self, open_flags: u32) {
        self.open_flags = open_flags;
    }

    /// Print the FuseCreateIn structure.
    pub fn print(&self) {
        debug!(
            "FuseCreateIn: flags: {:#x}, mode: {:#x}, umask: {:?}, open_flags: {:#x}",
            self.flags, self.mode, self.umask, self.open_flags
        );
    }

    /// Write the FuseCreateIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.flags.to_le_bytes());
        buf[4..8].copy_from_slice(&self.mode.to_le_bytes());
        buf[8..12].copy_from_slice(&self.umask.to_le_bytes());
        buf[12..16].copy_from_slice(&self.open_flags.to_le_bytes());
    }
}

/// FUSE release input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseReleaseIn {
    // 24 bytes
    fh: u64,
    flags: u32,
    release_flags: u32,
    lock_owner: u64,
}

impl FuseReleaseIn {
    /// Create a new FuseReleaseIn structure.
    pub fn new(fh: u64, flags: u32, release_flags: u32, lock_owner: u64) -> Self {
        Self {
            fh,
            flags,
            release_flags,
            lock_owner,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the release flags.
    pub fn set_release_flags(&mut self, release_flags: u32) {
        self.release_flags = release_flags;
    }

    /// Set the lock owner.
    pub fn set_lock_owner(&mut self, lock_owner: u64) {
        self.lock_owner = lock_owner;
    }

    /// Print the FuseReleaseIn structure.
    pub fn print(&self) {
        debug!(
            "FuseReleaseIn: fh: {:#x}, flags: {:#x}, release_flags: {:#x}, lock_owner: {:?}",
            self.fh, self.flags, self.release_flags, self.lock_owner
        );
    }

    /// Write the FuseReleaseIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
        buf[12..16].copy_from_slice(&self.release_flags.to_le_bytes());
        buf[16..24].copy_from_slice(&self.lock_owner.to_le_bytes());
    }
}

/// FUSE flush input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseFlushIn {
    // 24 bytes
    fh: u64,
    unused: u32,
    padding: u32,
    lock_owner: u64,
}

impl FuseFlushIn {
    /// Create a new FuseFlushIn structure.
    pub fn new(fh: u64, unused: u32, padding: u32, lock_owner: u64) -> Self {
        Self {
            fh,
            unused,
            padding,
            lock_owner,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the lock owner.
    pub fn set_lock_owner(&mut self, lock_owner: u64) {
        self.lock_owner = lock_owner;
    }

    /// Print the FuseFlushIn structure.
    pub fn print(&self) {
        debug!(
            "FuseFlushIn: fh: {:#x}, unused: {:?}, padding: {:?}, lock_owner: {:?}",
            self.fh, self.unused, self.padding, self.lock_owner
        );
    }

    /// Write the FuseFlushIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..12].copy_from_slice(&self.unused.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
        buf[16..24].copy_from_slice(&self.lock_owner.to_le_bytes());
    }
}

/// FUSE statfs output structure
#[derive(Debug, Clone, Copy, Default)]
pub struct FuseStatfsOut {
    // 80 bytes
    st: FuseKstatfs,
}

impl FuseStatfsOut {
    /// Create a new FuseStatfsOut structure.
    pub fn new(st: FuseKstatfs) -> Self {
        Self { st }
    }

    /// Read a FuseStatfsOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            st: FuseKstatfs::read_from(buf),
        }
    }

    /// Get the FUSE kstatfs structure.
    pub fn get_kstatfs(&self) -> FuseKstatfs {
        self.st
    }

    /// Print the FuseStatfsOut structure.
    pub fn print(&self) {
        debug!("FuseStatfsOut: st: {:?}", self.st);
    }
}

/// FUSE forget input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseForgetIn {
    // 8 bytes
    nlookup: u64,
}

impl FuseForgetIn {
    /// Create a new FuseForgetIn structure.
    pub fn new(nlookup: u64) -> Self {
        Self { nlookup }
    }

    /// Set the number of lookups to forget.
    pub fn set_nlookup(&mut self, nlookup: u64) {
        self.nlookup = nlookup;
    }

    /// Print the FuseForgetIn structure.
    pub fn print(&self) {
        debug!("FuseForgetIn: nlookup: {:?}", self.nlookup);
    }

    /// Write the FuseForgetIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.nlookup.to_le_bytes());
    }
}

/// FUSE forget one input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseForgetOne {
    // 16 bytes
    nodeid: u64,
    nlookup: u64,
}

impl FuseForgetOne {
    /// Create a new FuseForgetOne structure.
    pub fn new(nodeid: u64, nlookup: u64) -> Self {
        Self { nodeid, nlookup }
    }

    /// Set the node ID.
    pub fn set_nodeid(&mut self, nodeid: u64) {
        self.nodeid = nodeid;
    }

    /// Set the number of lookups to forget.
    pub fn set_nlookup(&mut self, nlookup: u64) {
        self.nlookup = nlookup;
    }

    /// Read a FuseForgetOne structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            nodeid: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            nlookup: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
        }
    }

    /// Print the FuseForgetOne structure.
    pub fn print(&self) {
        debug!(
            "FuseForgetOne: nodeid: {:?}, nlookup: {:?}",
            self.nodeid, self.nlookup
        );
    }
}

/// FUSE batch forget input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseBatchForgetIn {
    // 8 bytes
    count: u32,
    dummy: u32,
}

impl FuseBatchForgetIn {
    /// Create a new FuseBatchForgetIn structure.
    pub fn new(count: u32) -> Self {
        Self { count, dummy: 0 }
    }

    /// Set the count of forget operations.
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    /// Print the FuseBatchForgetIn structure.
    pub fn print(&self) {
        debug!(
            "FuseBatchForgetIn: count: {:?}, dummy: {:?}",
            self.count, self.dummy
        );
    }

    /// Write the FuseBatchForgetIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.count.to_le_bytes());
        buf[4..8].copy_from_slice(&self.dummy.to_le_bytes());
    }
}

/// FUSE mknod input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseMknodIn {
    // 16 bytes
    mode: u32,
    rdev: u32,
    umask: u32,
    padding: u32,
}

impl FuseMknodIn {
    /// Create a new FuseMknodIn structure.
    pub fn new(mode: u32, rdev: u32, umask: u32) -> Self {
        Self {
            mode,
            rdev,
            umask,
            padding: 0,
        }
    }

    /// Set the mode of the file.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Set the device ID.
    pub fn set_rdev(&mut self, rdev: u32) {
        self.rdev = rdev;
    }

    /// Set the umask for the file.
    pub fn set_umask(&mut self, umask: u32) {
        self.umask = umask;
    }

    /// Print the FuseMknodIn structure.
    pub fn print(&self) {
        debug!(
            "FuseMknodIn: mode: {:#x}, rdev: {:#x}, umask: {:?}, padding: {:?}",
            self.mode, self.rdev, self.umask, self.padding
        );
    }

    /// Write the FuseMknodIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.mode.to_le_bytes());
        buf[4..8].copy_from_slice(&self.rdev.to_le_bytes());
        buf[8..12].copy_from_slice(&self.umask.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE mkdir input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseMkdirIn {
    // 8 bytes
    mode: u32,
    umask: u32,
}

impl FuseMkdirIn {
    /// Create a new FuseMkdirIn structure.
    pub fn new(mode: u32, umask: u32) -> Self {
        Self { mode, umask }
    }

    /// Set the mode of the directory.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Set the umask for the directory.
    pub fn set_umask(&mut self, umask: u32) {
        self.umask = umask;
    }

    /// Print the FuseMkdirIn structure.
    pub fn print(&self) {
        debug!(
            "FuseMkdirIn: mode: {:#x}, umask: {:?}",
            self.mode, self.umask
        );
    }

    /// Write the FuseMkdirIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.mode.to_le_bytes());
        buf[4..8].copy_from_slice(&self.umask.to_le_bytes());
    }
}

/// FUSE rename input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseRenameIn {
    // 8 bytes
    newdir: u64,
}

impl FuseRenameIn {
    /// Create a new FuseRenameIn structure.
    pub fn new(newdir: u64) -> Self {
        Self { newdir }
    }

    /// Set the new directory ID.
    pub fn set_newdir(&mut self, newdir: u64) {
        self.newdir = newdir;
    }

    /// Print the FuseRenameIn structure.
    pub fn print(&self) {
        debug!("FuseRenameIn: newdir: {:?}", self.newdir);
    }

    /// Write the FuseRenameIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.newdir.to_le_bytes());
    }
}

/// FUSE rename2 input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseRename2In {
    // 16 bytes
    newdir: u64,
    flags: u32,
    padding: u32,
}

impl FuseRename2In {
    /// Create a new FuseRename2In structure.
    pub fn new(newdir: u64, flags: u32) -> Self {
        Self {
            newdir,
            flags,
            padding: 0,
        }
    }

    /// Set the new directory ID.
    pub fn set_newdir(&mut self, newdir: u64) {
        self.newdir = newdir;
    }

    /// Set the flags for the rename operation.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Print the FuseRename2In structure.
    pub fn print(&self) {
        debug!(
            "FuseRename2In: newdir: {:?}, flags: {:#x}, padding: {:?}",
            self.newdir, self.flags, self.padding
        );
    }

    /// Write the FuseRename2In structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.newdir.to_le_bytes());
        buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE link input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseLinkIn {
    // 8 bytes
    oldnodeid: u64,
}

impl FuseLinkIn {
    /// Create a new FuseLinkIn structure.
    pub fn new(oldnodeid: u64) -> Self {
        Self { oldnodeid }
    }

    /// Set the old node ID.
    pub fn set_oldnodeid(&mut self, oldnodeid: u64) {
        self.oldnodeid = oldnodeid;
    }

    /// Print the FuseLinkIn structure.
    pub fn print(&self) {
        debug!("FuseLinkIn: oldnodeid: {:?}", self.oldnodeid);
    }

    /// Write the FuseLinkIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.oldnodeid.to_le_bytes());
    }
}

/// FUSE setxattr input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseSetxattrIn {
    // 16 bytes
    size: u32,
    flags: u32,
    setxattr_flags: u32,
    padding: u32,
}

impl FuseSetxattrIn {
    /// Create a new FuseSetxattrIn structure.
    pub fn new(size: u32, flags: u32, setxattr_flags: u32) -> Self {
        Self {
            size,
            flags,
            setxattr_flags,
            padding: 0,
        }
    }

    /// Set the size of the extended attribute.
    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    /// Set the flags for the extended attribute.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the setxattr flags.
    pub fn set_setxattr_flags(&mut self, setxattr_flags: u32) {
        self.setxattr_flags = setxattr_flags;
    }

    /// Print the FuseSetxattrIn structure.
    pub fn print(&self) {
        debug!(
            "FuseSetxattrIn: size: {:?}, flags: {:#x}, setxattr_flags: {:#x}, padding: {:?}",
            self.size, self.flags, self.setxattr_flags, self.padding
        );
    }

    /// Write the FuseSetxattrIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.size.to_le_bytes());
        buf[4..8].copy_from_slice(&self.flags.to_le_bytes());
        buf[8..12].copy_from_slice(&self.setxattr_flags.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE getxattr output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseGetxattrIn {
    // 8 bytes
    size: u32,
    padding: u32,
}

impl FuseGetxattrIn {
    /// Create a new FuseGetxattrIn structure.
    pub fn new(size: u32) -> Self {
        Self { size, padding: 0 }
    }

    /// Set the size of the extended attribute.
    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    /// Print the FuseGetxattrIn structure.
    pub fn print(&self) {
        debug!(
            "FuseGetxattrIn: size: {:?}, padding: {:?}",
            self.size, self.padding
        );
    }

    /// Write the FuseGetxattrIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.size.to_le_bytes());
        buf[4..8].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE getxattr output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseGetxattrOut {
    // 8 bytes
    size: u32,
    padding: u32,
}

impl FuseGetxattrOut {
    /// Create a new FuseGetxattrOut structure.
    pub fn new(size: u32) -> Self {
        Self { size, padding: 0 }
    }

    /// Get the size of the extended attribute.
    pub fn get_size(&self) -> u32 {
        self.size
    }

    /// Read a FuseGetxattrOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        }
    }

    /// Print the FuseGetxattrOut structure.
    pub fn print(&self) {
        debug!(
            "FuseGetxattrOut: size: {:?}, padding: {:?}",
            self.size, self.padding
        );
    }
}

/// FUSE access input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseAccessIn {
    // 8 bytes
    mask: u32,
    padding: u32,
}

impl FuseAccessIn {
    /// Create a new FuseAccessIn structure.
    pub fn new(mask: u32) -> Self {
        Self { mask, padding: 0 }
    }

    /// Set the access mask.
    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    /// Print the FuseAccessIn structure.
    pub fn print(&self) {
        debug!(
            "FuseAccessIn: mask: {:?}, padding: {:?}",
            self.mask, self.padding
        );
    }

    /// Write the FuseAccessIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.mask.to_le_bytes());
        buf[4..8].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE fsync input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseFsyncIn {
    // 8 bytes
    fh: u64,
    fsync_flags: u32,
    padding: u32,
}

impl FuseFsyncIn {
    /// Create a new FuseFsyncIn structure.
    pub fn new(fh: u64, fsync_flags: u32) -> Self {
        Self {
            fh,
            fsync_flags,
            padding: 0,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the fsync flags.
    pub fn set_fsync_flags(&mut self, fsync_flags: u32) {
        self.fsync_flags = fsync_flags;
    }

    /// Print the FuseFsyncIn structure.
    pub fn print(&self) {
        debug!(
            "FuseFsyncIn: fh: {:#x}, fsync_flags: {:#x}, padding: {:?}",
            self.fh, self.fsync_flags, self.padding
        );
    }

    /// Write the FuseFsyncIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..12].copy_from_slice(&self.fsync_flags.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE bmap input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseBmapIn {
    // 16 bytes
    block: u64,
    blocksize: u32,
    padding: u32,
}

impl FuseBmapIn {
    /// Create a new FuseBmapIn structure.
    pub fn new(block: u64, blocksize: u32) -> Self {
        Self {
            block,
            blocksize,
            padding: 0,
        }
    }

    /// Set the block number.
    pub fn set_block(&mut self, block: u64) {
        self.block = block;
    }

    /// Set the block size.
    pub fn set_blocksize(&mut self, blocksize: u32) {
        self.blocksize = blocksize;
    }

    /// Print the FuseBmapIn structure.
    pub fn print(&self) {
        debug!(
            "FuseBmapIn: block: {:?}, blocksize: {:?}, padding: {:?}",
            self.block, self.blocksize, self.padding
        );
    }

    /// Write the FuseBmapIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.block.to_le_bytes());
        buf[8..12].copy_from_slice(&self.blocksize.to_le_bytes());
        buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE bmap output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseBmapOut {
    // 8 bytes
    block: u64,
}

impl FuseBmapOut {
    /// Create a new FuseBmapOut structure.
    pub fn new(block: u64) -> Self {
        Self { block }
    }

    /// Read a FuseBmapOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            block: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
        }
    }

    /// Get the block number.
    pub fn get_block(&self) -> u64 {
        self.block
    }

    /// Print the FuseBmapOut structure.
    pub fn print(&self) {
        debug!("FuseBmapOut: block: {:?}", self.block);
    }
}

/// FUSE ioctl input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlIn {
    // 32 bytes
    fh: u64,
    flags: u32,
    cmd: u32,
    arg: u64,
    in_size: u32,
    out_size: u32,
}

impl FuseIoctlIn {
    /// Create a new FuseIoctlIn structure.
    pub fn new(fh: u64, flags: u32, cmd: u32, arg: u64, in_size: u32, out_size: u32) -> Self {
        Self {
            fh,
            flags,
            cmd,
            arg,
            in_size,
            out_size,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the command.
    pub fn set_cmd(&mut self, cmd: u32) {
        self.cmd = cmd;
    }

    /// Set the argument.
    pub fn set_arg(&mut self, arg: u64) {
        self.arg = arg;
    }

    /// Set the input size.
    pub fn set_in_size(&mut self, in_size: u32) {
        self.in_size = in_size;
    }

    /// Set the output size.
    pub fn set_out_size(&mut self, out_size: u32) {
        self.out_size = out_size;
    }

    /// Print the FuseIoctlIn structure.
    pub fn print(&self) {
        debug!("FuseIoctlIn: fh: {:#x}, flags: {:#x}, cmd: {:?}, arg: {:?}, in_size: {:?}, out_size: {:?}", self.fh, self.flags, self.cmd, self.arg, self.in_size, self.out_size);
    }

    /// Write the FuseIoctlIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
        buf[12..16].copy_from_slice(&self.cmd.to_le_bytes());
        buf[16..24].copy_from_slice(&self.arg.to_le_bytes());
        buf[24..28].copy_from_slice(&self.in_size.to_le_bytes());
        buf[28..32].copy_from_slice(&self.out_size.to_le_bytes());
    }
}

/// FUSE ioctl iovec structure
#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlIovec {
    // 16 bytes
    base: u64,
    len: u64,
}

impl FuseIoctlIovec {
    /// Create a new FuseIoctlIovec structure.
    pub fn new(base: u64, len: u64) -> Self {
        Self { base, len }
    }

    /// Read a FuseIoctlIovec structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            base: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            len: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
        }
    }

    /// Get the base address.
    pub fn get_base(&self) -> u64 {
        self.base
    }

    /// Get the length.
    pub fn get_len(&self) -> u64 {
        self.len
    }

    /// Print the FuseIoctlIovec structure.
    pub fn print(&self) {
        debug!("FuseIoctlIovec: base: {:?}, len: {:?}", self.base, self.len);
    }
}

/// FUSE ioctl output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlOut {
    // 16 bytes
    result: i32,
    flags: u32,
    in_iovs: u32,
    out_iovs: u32,
}

impl FuseIoctlOut {
    /// Create a new FuseIoctlOut structure.
    pub fn new(result: i32, flags: u32, in_iovs: u32, out_iovs: u32) -> Self {
        Self {
            result,
            flags,
            in_iovs,
            out_iovs,
        }
    }

    /// Read a FuseIoctlOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            result: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
            flags: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            in_iovs: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            out_iovs: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
        }
    }

    /// Get the result of the ioctl operation.
    pub fn get_result(&self) -> i32 {
        self.result
    }

    /// Get the flags.
    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    /// Get the number of input iovecs.
    pub fn get_in_iovs(&self) -> u32 {
        self.in_iovs
    }

    /// Get the number of output iovecs.
    pub fn get_out_iovs(&self) -> u32 {
        self.out_iovs
    }

    /// Print the FuseIoctlOut structure.
    pub fn print(&self) {
        debug!(
            "FuseIoctlOut: result: {:?}, flags: {:#x}, in_iovs: {:?}, out_iovs: {:?}",
            self.result, self.flags, self.in_iovs, self.out_iovs
        );
    }
}

/// FUSE poll input structure
#[derive(Debug, Clone, Copy)]
pub struct FusePollIn {
    // 24 bytes
    fh: u64,
    kh: u64,
    flags: u32,
    events: u32,
}

impl FusePollIn {
    /// Create a new FusePollIn structure.
    pub fn new(fh: u64, kh: u64, flags: u32, events: u32) -> Self {
        Self {
            fh,
            kh,
            flags,
            events,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the kernel handle.
    pub fn set_kh(&mut self, kh: u64) {
        self.kh = kh;
    }

    /// Set the flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the events.
    pub fn set_events(&mut self, events: u32) {
        self.events = events;
    }

    /// Print the FusePollIn structure.
    pub fn print(&self) {
        debug!(
            "FusePollIn: fh: {:#x}, kh: {:?}, flags: {:#x}, events: {:?}",
            self.fh, self.kh, self.flags, self.events
        );
    }

    /// Write the FusePollIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..16].copy_from_slice(&self.kh.to_le_bytes());
        buf[16..20].copy_from_slice(&self.flags.to_le_bytes());
        buf[20..24].copy_from_slice(&self.events.to_le_bytes());
    }
}

/// FUSE poll output structure
#[derive(Debug, Clone, Copy)]
pub struct FusePollOut {
    // 8 bytes
    revents: u32,
    padding: u32,
}

impl FusePollOut {
    /// Create a new FusePollOut structure.
    pub fn new(revents: u32) -> Self {
        Self {
            revents,
            padding: 0,
        }
    }

    /// Read a FusePollOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            revents: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        }
    }

    /// Get the revents.
    pub fn get_revents(&self) -> u32 {
        self.revents
    }

    /// Print the FusePollOut structure.
    pub fn print(&self) {
        debug!(
            "FusePollOut: revents: {:?}, padding: {:?}",
            self.revents, self.padding
        );
    }
}

/// FUSE lseek input structure
#[derive(Debug, Clone, Copy)]
pub struct FuseLseekIn {
    // 24 bytes
    fh: u64,
    offset: u64,
    whence: u32,
    padding: u32,
}

impl FuseLseekIn {
    /// Create a new FuseLseekIn structure.
    pub fn new(fh: u64, offset: u64, whence: u32) -> Self {
        Self {
            fh,
            offset,
            whence,
            padding: 0,
        }
    }

    /// Set the file handle.
    pub fn set_fh(&mut self, fh: u64) {
        self.fh = fh;
    }

    /// Set the offset.
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Set the whence value.
    pub fn set_whence(&mut self, whence: u32) {
        self.whence = whence;
    }

    /// Print the FuseLseekIn structure.
    pub fn print(&self) {
        debug!(
            "FuseLseekIn: fh: {:#x}, offset: {:?}, whence: {:?}, padding: {:?}",
            self.fh, self.offset, self.whence, self.padding
        );
    }

    /// Write the FuseLseekIn structure to a buffer.
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
        buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
        buf[16..20].copy_from_slice(&self.whence.to_le_bytes());
        buf[20..24].copy_from_slice(&self.padding.to_le_bytes());
    }
}

/// FUSE lseek output structure
#[derive(Debug, Clone, Copy)]
pub struct FuseLseekOut {
    // 8 bytes
    offset: u64,
}

impl FuseLseekOut {
    /// Create a new FuseLseekOut structure.
    pub fn new(offset: u64) -> Self {
        Self { offset }
    }

    /// Read a FuseLseekOut structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            offset: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
        }
    }

    /// Get the offset.
    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    /// Print the FuseLseekOut structure.
    pub fn print(&self) {
        debug!("FuseLseekOut: offset: {:?}", self.offset);
    }
}

/// FUSE directory entry structure
#[derive(Debug)]
pub struct FuseDirent {
    // 24 bytes + len
    ino: u64,
    off: u64,
    namelen: u32,
    type_: u32,
    name: String,
}

impl FuseDirent {
    /// Create a new FuseDirent structure.
    pub fn new(ino: u64, off: u64, namelen: u32, type_: u32, name: String) -> Self {
        Self {
            ino,
            off,
            namelen,
            type_,
            name,
        }
    }

    /// Read a FuseDirent structure from a buffer.
    pub fn read_from(buf: &[u8]) -> Self {
        let ino = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let off = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        let namelen = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let type_ = u32::from_le_bytes(buf[20..24].try_into().unwrap());
        let name = String::from_utf8(buf[24..24 + namelen as usize].to_vec()).unwrap();
        Self {
            ino,
            off,
            namelen,
            type_,
            name,
        }
    }

    /// Get the inode number.
    pub fn get_ino(&self) -> u64 {
        self.ino
    }

    /// Get the offset.
    pub fn get_off(&self) -> u64 {
        self.off
    }

    /// Get the name length.
    pub fn get_namelen(&self) -> u32 {
        self.namelen
    }

    /// Get the type of the directory entry.
    pub fn get_type(&self) -> u32 {
        self.type_
    }

    /// Get the name of the directory entry.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Convert the type to a VfsNodeType.
    pub fn get_type_as_vfsnodetype(&self) -> VfsNodeType {
        match self.type_ {
            1 => VfsNodeType::Fifo,
            2 => VfsNodeType::CharDevice,
            4 => VfsNodeType::Dir,
            6 => VfsNodeType::BlockDevice,
            8 => VfsNodeType::File,
            10 => VfsNodeType::SymLink,
            12 => VfsNodeType::Socket,
            _ => VfsNodeType::File,
        }
    }

    /// Get the length of the FuseDirent structure.
    pub fn get_len(&self) -> usize {
        let padding = (8 - (self.namelen % 8)) % 8;
        debug!(
            "FuseDirent: padding: {:?}, len: {:?}",
            padding,
            (self.namelen + padding) as usize
        );
        24 + (self.namelen + padding) as usize
    }

    /// Print the FuseDirent structure.
    pub fn print(&self) {
        debug!(
            "FuseDirent: ino: {:?}, off: {:?}, namelen: {:?}, type: {:?}, name: {:?}",
            self.ino, self.off, self.namelen, self.type_, self.name
        );
    }
}
