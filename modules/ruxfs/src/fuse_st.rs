/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![allow(dead_code)]

use alloc::string::String;
use alloc::fmt::Debug;
use alloc::fmt::Formatter;
use alloc::fmt::Error;
use axfs_vfs::VfsNodeType;

#[derive(Debug, Clone, Copy)]
pub enum FuseOpcode {
	FuseLookup		= 1,
	FuseForget		= 2,  /* no reply */
	FuseGetattr		= 3,
	FuseSetattr		= 4,
	FuseReadlink	= 5,
	FuseSymlink		= 6,
	FuseMknod		= 8,
	FuseMkdir		= 9,
	FuseUnlink		= 10,
	FuseRmdir		= 11,
	FuseRename		= 12,
	FuseLink		= 13,
	FuseOpen		= 14,
	FuseRead		= 15,
	FuseWrite		= 16,
	FuseStatfs		= 17,
	FuseRelease		= 18,
	FuseFsync		= 20,
	FuseSetxattr		= 21,
	FuseGetxattr		= 22,
	FuseListxattr		= 23,
	FuseRemovexattr		= 24,
	FuseFlush		    = 25,
	FuseInit		    = 26,
	FuseOpendir			= 27,
	FuseReaddir			= 28,
	FuseReleasedir		= 29,
	FuseFsyncdir		= 30,
	FuseGetlk		    = 31,
	FuseSetlk		    = 32,
	FuseSetlkw		    = 33,
	FuseAccess		    = 34,
	FuseCreate		    = 35,
	FuseInterrupt	    = 36,
	FuseBmap		    = 37,
	FuseDestroy	    	= 38,
	FuseIoctl		    = 39,
	FusePoll		    = 40,
	FuseNotifyReply		= 41,
	FuseBatchForget		= 42,
	FuseFallocate		= 43,
	FuseReaddirplus		= 44,
	FuseRename2			= 45,
	FuseLseek		    = 46,
	FuseCopyFileRange	= 47,
	FuseSetupmapping	= 48,
	FuseRemovemapping	= 49,
	FuseSyncfs		    = 50,
	FuseTmpfile			= 51,
}

pub mod fuse_open_flags {
    pub const FOPEN_DIRECT_IO: u32 = 1 << 0;
    pub const FOPEN_KEEP_CACHE: u32 = 1 << 1;
    pub const FOPEN_NONSEEKABLE: u32 = 1 << 2;
    pub const FOPEN_CACHE_DIR: u32 = 1 << 3;
    pub const FOPEN_STREAM: u32 = 1 << 4;
    pub const FOPEN_NOFLUSH: u32 = 1 << 5;
    pub const FOPEN_PARALLEL_DIRECT_WRITES: u32 = 1 << 6;
}

pub mod fuse_setattr_bitmasks {
    // Bitmasks for fuse_setattr_in.valid
    pub const FATTR_MODE: u32 = 1 << 0;
    pub const FATTR_UID: u32 = 1 << 1;
    pub const FATTR_GID: u32 = 1 << 2;
    pub const FATTR_SIZE: u32 = 1 << 3;
    pub const FATTR_ATIME: u32 = 1 << 4;
    pub const FATTR_MTIME: u32 = 1 << 5;
    pub const FATTR_FH: u32 = 1 << 6;
    pub const FATTR_ATIME_NOW: u32 = 1 << 7;
    pub const FATTR_MTIME_NOW: u32 = 1 << 8;
    pub const FATTR_LOCKOWNER: u32 = 1 << 9;
    pub const FATTR_CTIME: u32 = 1 << 10;
    pub const FATTR_KILL_SUIDGID: u32 = 1 << 11;
}

pub mod fuse_init_flags {
    pub const FUSE_ASYNC_READ: u32 = 1 << 0;
    pub const FUSE_POSIX_LOCKS: u32 = 1 << 1;
    pub const FUSE_FILE_OPS: u32 = 1 << 2;
    pub const FUSE_ATOMIC_O_TRUNC: u32 = 1 << 3;
    pub const FUSE_EXPORT_SUPPORT: u32 = 1 << 4;
    pub const FUSE_BIG_WRITES: u32 = 1 << 5;
    pub const FUSE_DONT_MASK: u32 = 1 << 6;
    pub const FUSE_SPLICE_WRITE: u32 = 1 << 7;
    pub const FUSE_SPLICE_MOVE: u32 = 1 << 8;
    pub const FUSE_SPLICE_READ: u32 = 1 << 9;
    pub const FUSE_FLOCK_LOCKS: u32 = 1 << 10;
    pub const FUSE_HAS_IOCTL_DIR: u32 = 1 << 11;
    pub const FUSE_AUTO_INVAL_DATA: u32 = 1 << 12;
    pub const FUSE_DO_READDIRPLUS: u32 = 1 << 13;
    pub const FUSE_READDIRPLUS_AUTO: u32 = 1 << 14;
    pub const FUSE_ASYNC_DIO: u32 = 1 << 15;
    pub const FUSE_WRITEBACK_CACHE: u32 = 1 << 16;
    pub const FUSE_NO_OPEN_SUPPORT: u32 = 1 << 17;
    pub const FUSE_PARALLEL_DIROPS: u32 = 1 << 18;
    pub const FUSE_HANDLE_KILLPRIV: u32 = 1 << 19;
    pub const FUSE_POSIX_ACL: u32 = 1 << 20;
    pub const FUSE_ABORT_ERROR: u32 = 1 << 21;
    pub const FUSE_MAX_PAGES: u32 = 1 << 22;
    pub const FUSE_CACHE_SYMLINKS: u32 = 1 << 23;
    pub const FUSE_NO_OPENDIR_SUPPORT: u32 = 1 << 24;
    pub const FUSE_EXPLICIT_INVAL_DATA: u32 = 1 << 25;
    pub const FUSE_MAP_ALIGNMENT: u32 = 1 << 26;
    pub const FUSE_SUBMOUNTS: u32 = 1 << 27;
    pub const FUSE_HANDLE_KILLPRIV_V2: u32 = 1 << 28;
    pub const FUSE_SETXATTR_EXT: u32 = 1 << 29;
    pub const FUSE_INIT_EXT: u32 = 1 << 30;
    pub const FUSE_INIT_RESERVED: u32 = 1 << 31;
    pub const FUSE_SECURITY_CTX: u64 = 1 << 32;
    pub const FUSE_HAS_INODE_DAX: u64 = 1 << 33;
    pub const FUSE_CREATE_SUPP_GROUP: u64 = 1 << 34;
}

pub mod release_flags {
    pub const FUSE_RELEASE_FLUSH: u32 = 1 << 0;
    pub const FUSE_RELEASE_FLOCK_UNLOCK: u32 = 1 << 1;
}

pub mod getattr_flags {
    pub const FUSE_GETATTR_FH: u32 = 1 << 0;
}

pub mod write_flags {
    pub const FUSE_WRITE_CACHE: u32 = 1 << 0;
    pub const FUSE_WRITE_LOCKOWNER: u32 = 1 << 1;
    pub const FUSE_WRITE_KILL_SUIDGID: u32 = 1 << 2;
}

pub mod read_flags {
    pub const FUSE_READ_LOCKOWNER: u32 = 1 << 1;
}

#[derive(Debug, Clone, Copy)]
pub struct FuseInHeader { // 40 bytes
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
    pub fn new(len: u32, opcode: u32, unique: u64, nodeid: u64, uid: u32, gid: u32, pid: u32) -> Self {
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

	pub fn set_len(&mut self, len: u32) {
		self.len = len;
	}

	pub fn set_opcode(&mut self, opcode: u32) {
		self.opcode = opcode;
	}

	pub fn set_unique(&mut self, unique: u64) {
		self.unique = unique;
	}

	pub fn set_nodeid(&mut self, nodeid: u64) {
		self.nodeid = nodeid;
	}

	pub fn set_uid(&mut self, uid: u32) {
		self.uid = uid;
	}

	pub fn set_gid(&mut self, gid: u32) {
		self.gid = gid;
	}

	pub fn set_pid(&mut self, pid: u32) {
		self.pid = pid;
	}

	pub fn print(&self) {
		info!("FuseInHeader: len: {:?}, opcode: {:?}, unique: {:?}, nodeid: {:?}, uid: {:?}, gid: {:?}, pid: {:?}, padding: {:?}", self.len, self.opcode, self.unique, self.nodeid, self.uid, self.gid, self.pid, self.padding);
	}

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

#[derive(Debug, Clone, Copy)]
pub struct FuseOutHeader { // 16 bytes
    len: u32,     // length of the response
    error: i32,   // error code
    unique: u64,  // unique request ID
}

impl FuseOutHeader {
    pub fn new(len: u32, error: i32, unique: u64) -> Self {
        Self {
            len,
            error,
            unique,
        }
    }

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			len: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
			error: i32::from_le_bytes(buf[4..8].try_into().unwrap()),
			unique: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
		}
	}

	pub fn is_ok(&self) -> bool {
		self.error == 0
	}

	pub fn get_len(&self) -> u32 {
		self.len
	}

	pub fn error(&self) -> i32 {
		self.error
	}

	pub fn get_unique(&self) -> u64 {
		self.unique
	}

	pub fn print(&self) {
		info!("fuse_out_header: len: {:?}, error: {:?}, unique: {:?}", self.len, self.error, self.unique);
	}
    
	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.len.to_le_bytes());
		buf[4..8].copy_from_slice(&self.error.to_le_bytes());
		buf[8..16].copy_from_slice(&self.unique.to_le_bytes());
	}
}


#[derive(Debug, Clone, Copy)]
pub struct FuseInitIn { // 64 bytes
	major: u32,
	minor: u32,
    max_readahead: u32,
    flags: u32,
    flags2: u32,
    unused: [u32; 11],
}

impl FuseInitIn {
    pub fn new(major: u32, minor: u32, max_readahead: u32, flags: u32, flags2: u32, unused: [u32; 11]) -> Self {
        Self {
            major,
            minor,
            max_readahead,
            flags,
            flags2,
            unused,
        }
    }

	pub fn print(&self) {
		info!("FuseInitIn: major: {:?}, minor: {:?}, max_readahead: {:#x}, flags: {:#x}, flags2: {:?}, unused: {:?}", self.major, self.minor, self.max_readahead, self.flags, self.flags2, self.unused);
	}

    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.major.to_le_bytes());
        buf[4..8].copy_from_slice(&self.minor.to_le_bytes());
        buf[8..12].copy_from_slice(&self.max_readahead.to_le_bytes());
        buf[12..16].copy_from_slice(&self.flags.to_le_bytes());
        buf[16..20].copy_from_slice(&self.flags2.to_le_bytes());
		for (i, &val) in self.unused.iter().enumerate() {
			buf[20 + i * 4..24 + i * 4].copy_from_slice(&val.to_le_bytes());
		}
        // buf[20..52].copy_from_slice(&self.unused.to_le_bytes());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FuseInitOut { // 64 bytes
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
	pub fn new(major: u32, minor: u32, max_readahead: u32, flags: u32, max_background: u16, congestion_threshold: u16, max_write: u32, time_gran: u32, max_pages: u16, map_alignment: u16, flags2: u32, unused: [u32; 7]) -> Self {
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

	pub fn read_from(buf: &[u8]) -> Self {
		debug!("fuseinitout from len: {:?}, buf: {:?}", buf.len(), buf);
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

	pub fn get_major(&self) -> u32 {
		self.major
	}

	pub fn get_minor(&self) -> u32 {
		self.minor
	}

	pub fn get_max_readahead(&self) -> u32 {
		self.max_readahead
	}

	pub fn get_flags(&self) -> u32 {
		self.flags
	}

	pub fn get_max_background(&self) -> u16 {
		self.max_background
	}

	pub fn get_congestion_threshold(&self) -> u16 {
		self.congestion_threshold
	}

	pub fn get_max_write(&self) -> u32 {
		self.max_write
	}

	pub fn get_time_gran(&self) -> u32 {
		self.time_gran
	}

	pub fn get_max_pages(&self) -> u16 {
		self.max_pages
	}

	pub fn get_map_alignment(&self) -> u16 {
		self.map_alignment
	}

	pub fn get_flags2(&self) -> u32 {
		self.flags2
	}

	pub fn print(&self) {
		info!("FuseInitOut: major: {:?}, minor: {:?}, max_readahead: {:#x}, flags: {:#x}, max_background: {:?}, congestion_threshold: {:?}, max_write: {:#x}, time_gran: {:?}, max_pages: {:?}, map_alignment: {:?}, flags2: {:?}, unused: {:?}", self.major, self.minor, self.max_readahead, self.flags, self.max_background, self.congestion_threshold, self.max_write, self.time_gran, self.max_pages, self.map_alignment, self.flags2, self.unused);
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseGetattrIn { // 16 bytes
	getattr_flags: u32,
	dummy: u32,
	fh: u64,
}

impl FuseGetattrIn {
	pub fn new(getattr_flags: u32, dummy: u32, fh: u64) -> Self {
		Self {
			getattr_flags,
			dummy,
			fh,
		}
	}

	pub fn print(&self) {
		info!("FuseGetattrIn: getattr_flags: {:#x}, dummy: {:?}, fh: {:#x}", self.getattr_flags, self.dummy, self.fh);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.getattr_flags.to_le_bytes());
		buf[4..8].copy_from_slice(&self.dummy.to_le_bytes());
		buf[8..16].copy_from_slice(&self.fh.to_le_bytes());
	}
}

#[derive(Clone, Copy)]
pub struct FuseAttr { // 88 bytes
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
	pub fn new(ino: u64, size: u64, blocks: u64, atime: u64, mtime: u64, ctime: u64, atimensec: u32, mtimensec: u32, ctimensec: u32, mode: u32, nlink: u32, uid: u32, gid: u32, rdev: u32, blksize: u32, flags: u32) -> Self {
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

	pub fn get_size(&self) -> u64 {
		self.size
	}

	pub fn get_mode(&self) -> u32 {
		self.mode
	}

	pub fn get_uid(&self) -> u32 {
		self.uid
	}

	pub fn get_gid(&self) -> u32 {
		self.gid
	}

	pub fn get_nlink(&self) -> u32 {
		self.nlink
	}

	pub fn get_ino(&self) -> u64 {
		self.ino
	}

	pub fn get_blocks(&self) -> u64 {
		self.blocks
	}

	pub fn get_atime(&self) -> u64 {
		self.atime
	}

	pub fn get_mtime(&self) -> u64 {
		self.mtime
	}

	pub fn get_ctime(&self) -> u64 {
		self.ctime
	}

	pub fn get_atimensec(&self) -> u32 {
		self.atimensec
	}

	pub fn get_mtimensec(&self) -> u32 {
		self.mtimensec
	}

	pub fn get_ctimensec(&self) -> u32 {
		self.ctimensec
	}

	pub fn get_rdev(&self) -> u32 {
		self.rdev
	}

	pub fn get_blksize(&self) -> u32 {
		self.blksize
	}

	pub fn get_flags(&self) -> u32 {
		self.flags
	}

	pub fn set_size(&mut self, size: u64) {
		self.size = size;
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.mode = mode;
	}

	pub fn set_uid(&mut self, uid: u32) {
		self.uid = uid;
	}

	pub fn set_gid(&mut self, gid: u32) {
		self.gid = gid;
	}

	pub fn set_nlink(&mut self, nlink: u32) {
		self.nlink = nlink;
	}

	pub fn set_ino(&mut self, ino: u64) {
		self.ino = ino;
	}

	pub fn set_blocks(&mut self, blocks: u64) {
		self.blocks = blocks;
	}

	pub fn set_atime(&mut self, atime: u64) {
		self.atime = atime;
	}

	pub fn set_mtime(&mut self, mtime: u64) {
		self.mtime = mtime;
	}

	pub fn set_ctime(&mut self, ctime: u64) {
		self.ctime = ctime;
	}

	pub fn set_atimensec(&mut self, atimensec: u32) {
		self.atimensec = atimensec;
	}

	pub fn set_mtimensec(&mut self, mtimensec: u32) {
		self.mtimensec = mtimensec;
	}

	pub fn set_ctimensec(&mut self, ctimensec: u32) {
		self.ctimensec = ctimensec;
	}

	pub fn set_rdev(&mut self, rdev: u32) {
		self.rdev = rdev;
	}

	pub fn set_blksize(&mut self, blksize: u32) {
		self.blksize = blksize;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn print(&self) {
		info!("FuseAttr: ino: {:?}, size: {:?}, blocks: {:?}, atime: {:?}, mtime: {:?}, ctime: {:?}, atimensec: {:?}, mtimensec: {:?}, ctimensec: {:?}, mode: {:#x}, nlink: {:?}, uid: {:?}, gid: {:?}, rdev: {:#x}, blksize: {:?}, flags: {:#x}", self.ino, self.size, self.blocks, self.atime, self.mtime, self.ctime, self.atimensec, self.mtimensec, self.ctimensec, self.mode, self.nlink, self.uid, self.gid, self.rdev, self.blksize, self.flags);
	}

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

impl Default for FuseAttr {
	fn default() -> Self {
		Self {
			ino: 0,
			size: 0,
			blocks: 0,
			atime: 0,
			mtime: 0,
			ctime: 0,
			atimensec: 0,
			mtimensec: 0,
			ctimensec: 0,
			mode: 0,
			nlink: 0,
			uid: 0,
			gid: 0,
			rdev: 0,
			blksize: 0,
			flags: 0,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseKstatfs { // 80 bytes
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
	pub fn new(blocks: u64, bfree: u64, bavail: u64, files: u64, ffree: u64, bsize: u32, namelen: u32, frsize: u32, padding: u32, spare: [u32; 6]) -> Self {
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

	pub fn print(&self) {
		info!("FuseKstatfs: blocks: {:?}, bfree: {:?}, bavail: {:?}, files: {:?}, ffree: {:?}, bsize: {:?}, namelen: {:?}, frsize: {:?}, padding: {:?}, spare: {:?}", self.blocks, self.bfree, self.bavail, self.files, self.ffree, self.bsize, self.namelen, self.frsize, self.padding, self.spare);
	}
}

impl Default for FuseKstatfs {
	fn default() -> Self {
		Self {
			blocks: 0,
			bfree: 0,
			bavail: 0,
			files: 0,
			ffree: 0,
			bsize: 0,
			namelen: 0,
			frsize: 0,
			padding: 0,
			spare: [0; 6],
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseFileLock { // 24 bytes
	start: u64,
	end: u64,
	type_: u32,
	pid: u32,
}

impl FuseFileLock {
	pub fn new(start: u64, end: u64, type_: u32, pid: u32) -> Self {
		Self {
			start,
			end,
			type_,
			pid,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			start: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
			end: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
			type_: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
			pid: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
		}
	}

	pub fn print(&self) {
		info!("FuseFileLock: start: {:?}, end: {:?}, type: {:?}, pid: {:?}", self.start, self.end, self.type_, self.pid);
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseAttrOut { // 104 bytes
	attr_valid: u64,
	attr_valid_nsec: u32,
	dummy: u32,
	attr: FuseAttr,
}

impl FuseAttrOut {
	pub fn new(attr_valid: u64, attr_valid_nsec: u32, dummy: u32, attr: FuseAttr) -> Self {
		Self {
			attr_valid,
			attr_valid_nsec,
			dummy,
			attr,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			attr_valid: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
			attr_valid_nsec: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
			dummy: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
			attr: FuseAttr::read_from(&buf[16..104]),
		}
	}

	pub fn get_attr_valid(&self) -> u64 {
		self.attr_valid
	}

	pub fn get_attr_valid_nsec(&self) -> u32 {
		self.attr_valid_nsec
	}

	pub fn get_dummy(&self) -> u32 {
		self.dummy
	}

	pub fn get_attr(&self) -> FuseAttr {
		self.attr
	}

	pub fn get_size(&self) -> u64 {
		self.attr.size
	}

	pub fn print(&self) {
		info!("FuseAttrOut: attr_valid: {:?}, attr_valid_nsec: {:?}, dummy: {:?}, attr: {:?}", self.attr_valid, self.attr_valid_nsec, self.dummy, self.attr);
	}
}

impl Default for FuseAttrOut {
	fn default() -> Self {
		Self {
			attr_valid: 0,
			attr_valid_nsec: 0,
			dummy: 0,
			attr: FuseAttr::default(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseEntryOut { // 128 bytes
	nodeid: u64,		/* Inode ID */
	generation: u64,	/* Inode generation: nodeid:gen must
					   be unique for the fs's lifetime */
	entry_valid: u64,	/* Cache timeout for the name */
	attr_valid: u64,	/* Cache timeout for the attributes */
	entry_valid_nsec: u32,
	attr_valid_nsec: u32,
	attr: FuseAttr,
}

impl FuseEntryOut {
	pub fn new(nodeid: u64, generation: u64, entry_valid: u64, attr_valid: u64, entry_valid_nsec: u32, attr_valid_nsec: u32, attr: FuseAttr) -> Self {
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

	pub fn get_nodeid(&self) -> u64 {
		self.nodeid
	}

	pub fn get_generation(&self) -> u64 {
		self.generation
	}

	pub fn get_entry_valid(&self) -> u64 {
		self.entry_valid
	}

	pub fn get_attr_valid(&self) -> u64 {
		self.attr_valid
	}

	pub fn get_entry_valid_nsec(&self) -> u32 {
		self.entry_valid_nsec
	}

	pub fn get_attr_valid_nsec(&self) -> u32 {
		self.attr_valid_nsec
	}

	pub fn get_attr(&self) -> FuseAttr {
		self.attr
	}

	pub fn get_nlink(&self) -> u32 {
		self.attr.nlink
	}

	pub fn get_size(&self) -> u64 {
		self.attr.size
	}

	pub fn print(&self) {
		info!("FuseEntryOut: nodeid: {:?}, generation: {:?}, entry_valid: {:?}, attr_valid: {:?}, entry_valid_nsec: {:?}, attr_valid_nsec: {:?}, attr: {:?}", self.nodeid, self.generation, self.entry_valid, self.attr_valid, self.entry_valid_nsec, self.attr_valid_nsec, self.attr);
	}
}

impl Default for FuseEntryOut {
	fn default() -> Self {
		Self {
			nodeid: 0,
			generation: 0,
			entry_valid: 0,
			attr_valid: 0,
			entry_valid_nsec: 0,
			attr_valid_nsec: 0,
			attr: FuseAttr::default(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseSetattrIn { // 88 bytes
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
	pub fn new(valid: u32, padding: u32, fh: u64, size: u64, lock_owner: u64, atime: u64, mtime: u64, ctime: u64, atimensec: u32, mtimensec: u32, ctimensec: u32, mode: u32, unused4: u32, uid: u32, gid: u32, unused5: u32) -> Self {
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

	pub fn set_valid(&mut self, valid: u32) {
		self.valid = valid;
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_size(&mut self, size: u64) {
		self.size = size;
	}

	pub fn set_lock_owner(&mut self, lock_owner: u64) {
		self.lock_owner = lock_owner;
	}

	pub fn set_atime(&mut self, atime: u64) {
		self.atime = atime;
	}

	pub fn set_mtime(&mut self, mtime: u64) {
		self.mtime = mtime;
	}

	pub fn set_ctime(&mut self, ctime: u64) {
		self.ctime = ctime;
	}

	pub fn set_atimensec(&mut self, atimensec: u32) {
		self.atimensec = atimensec;
	}

	pub fn set_mtimensec(&mut self, mtimensec: u32) {
		self.mtimensec = mtimensec;
	}

	pub fn set_ctimensec(&mut self, ctimensec: u32) {
		self.ctimensec = ctimensec;
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.mode = mode;
	}

	pub fn set_uid(&mut self, uid: u32) {
		self.uid = uid;
	}

	pub fn set_gid(&mut self, gid: u32) {
		self.gid = gid;
	}

	pub fn print(&self) {
		info!("FuseSetattrIn: valid: {:?}, padding: {:?}, fh: {:#x}, size: {:?}, lock_owner: {:?}, atime: {:?}, mtime: {:?}, ctime: {:?}, atimensec: {:?}, mtimensec: {:?}, ctimensec: {:?}, mode: {:#x}, unused4: {:?}, uid: {:?}, gid: {:?}, unused5: {:?}", self.valid, self.padding, self.fh, self.size, self.lock_owner, self.atime, self.mtime, self.ctime, self.atimensec, self.mtimensec, self.ctimensec, self.mode, self.unused4, self.uid, self.gid, self.unused5);
	}

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

#[derive(Debug, Clone, Copy)]
pub struct FuseOpenIn { // 8 bytes
	flags: u32,
	open_flags: u32,	/* FUSE_OPEN_... */
}

impl FuseOpenIn {
	pub fn new(flags: u32, open_flags: u32) -> Self {
		Self {
			flags,
			open_flags,
		}
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_open_flags(&mut self, open_flags: u32) {
		self.open_flags = open_flags;
	}

	pub fn print(&self) {
		info!("FuseOpenIn: flags: {:#x}, open_flags: {:#x}", self.flags, self.open_flags);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.flags.to_le_bytes());
		buf[4..8].copy_from_slice(&self.open_flags.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseOpenOut { // 16 bytes
	fh: u64,
	open_flags: u32,
	padding: u32,
}

impl FuseOpenOut {
	pub fn new(fh: u64, open_flags: u32) -> Self {
		Self {
			fh,
			open_flags,
			padding: 0,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			fh: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
			open_flags: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
			padding: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
		}
	}

	pub fn get_fh(&self) -> u64 {
		self.fh
	}

	pub fn get_open_flags(&self) -> u32 {
		self.open_flags
	}

	pub fn print(&self) {
		info!("FuseOpenOut: fh: {:#x}, open_flags: {:#x}, padding: {:?}", self.fh, self.open_flags, self.padding);
	}
}

impl Default for FuseOpenOut {
	fn default() -> Self {
		Self {
			fh: 0,
			open_flags: 0,
			padding: 0,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseReadIn { // 40 bytes
	fh: u64,
	offset: u64,
	size: u32,
	read_flags: u32,
	lock_owner: u64,
	flags: u32,
	padding: u32,
}

impl FuseReadIn {
	pub fn new(fh: u64, offset: u64, size: u32, read_flags: u32, lock_owner: u64, flags: u32) -> Self {
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

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.offset = offset;
	}

	pub fn set_size(&mut self, size: u32) {
		self.size = size;
	}

	pub fn set_read_flags(&mut self, read_flags: u32) {
		self.read_flags = read_flags;
	}

	pub fn set_lock_owner(&mut self, lock_owner: u64) {
		self.lock_owner = lock_owner;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn print(&self) {
		info!("FuseReadIn: fh: {:#x}, offset: {:?}, size: {:?}, read_flags: {:#x}, lock_owner: {:?}, flags: {:#x}, padding: {:?}", self.fh, self.offset, self.size, self.read_flags, self.lock_owner, self.flags, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
		buf[16..20].copy_from_slice(&self.size.to_le_bytes());
		buf[20..24].copy_from_slice(&self.read_flags.to_le_bytes());
		buf[24..32].copy_from_slice(&self.lock_owner.to_le_bytes());
		buf[32..36].copy_from_slice(&self.flags.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseWriteIn { // 40 bytes
	fh: u64,
	offset: u64,
	size: u32,
	write_flags: u32,
	lock_owner: u64,
	flags: u32,
	padding: u32,
}

impl FuseWriteIn {
	pub fn new(fh: u64, offset: u64, size: u32, write_flags: u32, lock_owner: u64, flags: u32) -> Self {
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

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.offset = offset;
	}

	pub fn set_size(&mut self, size: u32) {
		self.size = size;
	}

	pub fn set_write_flags(&mut self, write_flags: u32) {
		self.write_flags = write_flags;
	}

	pub fn set_lock_owner(&mut self, lock_owner: u64) {
		self.lock_owner = lock_owner;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn print(&self) {
		info!("FuseWriteIn: fh: {:#x}, offset: {:?}, size: {:?}, write_flags: {:#x}, lock_owner: {:?}, flags: {:#x}, padding: {:?}", self.fh, self.offset, self.size, self.write_flags, self.lock_owner, self.flags, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
		buf[16..20].copy_from_slice(&self.size.to_le_bytes());
		buf[20..24].copy_from_slice(&self.write_flags.to_le_bytes());
		buf[24..32].copy_from_slice(&self.lock_owner.to_le_bytes());
		buf[32..36].copy_from_slice(&self.flags.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseWriteOut { // 8 bytes
	size: u32,
	padding: u32,
}

impl FuseWriteOut {
	pub fn new(size: u32) -> Self {
		Self {
			size,
			padding: 0,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
			padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
		}
	}

	pub fn get_size(&self) -> u32 {
		self.size
	}

	pub fn print(&self) {
		info!("FuseWriteOut: size: {:?}, padding: {:?}", self.size, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.size.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseCreateIn { // 16 bytes
	flags: u32,
	mode: u32,
	umask: u32,
	open_flags: u32,	/* FUSE_OPEN_... */
}

impl FuseCreateIn {
	pub fn new(flags: u32, mode: u32, umask: u32, open_flags: u32) -> Self {
		Self {
			flags,
			mode,
			umask,
			open_flags,
		}
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.mode = mode;
	}

	pub fn set_umask(&mut self, umask: u32) {
		self.umask = umask;
	}

	pub fn set_open_flags(&mut self, open_flags: u32) {
		self.open_flags = open_flags;
	}

	pub fn print(&self) {
		info!("FuseCreateIn: flags: {:#x}, mode: {:#x}, umask: {:?}, open_flags: {:#x}", self.flags, self.mode, self.umask, self.open_flags);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.flags.to_le_bytes());
		buf[4..8].copy_from_slice(&self.mode.to_le_bytes());
		buf[8..12].copy_from_slice(&self.umask.to_le_bytes());
		buf[12..16].copy_from_slice(&self.open_flags.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseReleaseIn { // 24 bytes
	fh: u64,
	flags: u32,
	release_flags: u32,
	lock_owner: u64,
}

impl FuseReleaseIn {
	pub fn new(fh: u64, flags: u32, release_flags: u32, lock_owner: u64) -> Self {
		Self {
			fh,
			flags,
			release_flags,
			lock_owner,
		}
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_release_flags(&mut self, release_flags: u32) {
		self.release_flags = release_flags;
	}

	pub fn set_lock_owner(&mut self, lock_owner: u64) {
		self.lock_owner = lock_owner;
	}

	pub fn print(&self) {
		info!("FuseReleaseIn: fh: {:#x}, flags: {:#x}, release_flags: {:#x}, lock_owner: {:?}", self.fh, self.flags, self.release_flags, self.lock_owner);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
		buf[12..16].copy_from_slice(&self.release_flags.to_le_bytes());
		buf[16..24].copy_from_slice(&self.lock_owner.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseFlushIn { // 24 bytes
	fh: u64,
	unused: u32,
	padding: u32,
	lock_owner: u64,
}

impl FuseFlushIn {
	pub fn new(fh: u64, unused: u32, padding: u32, lock_owner: u64) -> Self {
		Self {
			fh,
			unused,
			padding,
			lock_owner,
		}
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_lock_owner(&mut self, lock_owner: u64) {
		self.lock_owner = lock_owner;
	}

	pub fn print(&self) {
		info!("FuseFlushIn: fh: {:#x}, unused: {:?}, padding: {:?}, lock_owner: {:?}", self.fh, self.unused, self.padding, self.lock_owner);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..12].copy_from_slice(&self.unused.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
		buf[16..24].copy_from_slice(&self.lock_owner.to_le_bytes());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseStatfsOut { // 80 bytes
	st: FuseKstatfs,
}

impl FuseStatfsOut {
	pub fn new(st: FuseKstatfs) -> Self {
		Self {
			st,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			st: FuseKstatfs::read_from(buf),
		}
	}

	pub fn get_kstatfs(&self) -> FuseKstatfs {
		self.st
	}

	pub fn print(&self) {
		info!("FuseStatfsOut: st: {:?}", self.st);
	}
}

impl Default for FuseStatfsOut {
	fn default() -> Self {
		Self {
			st: FuseKstatfs::default(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseForgetIn { // 8 bytes
	nlookup: u64,
}

impl FuseForgetIn {
	pub fn new(nlookup: u64) -> Self {
		Self {
			nlookup,
		}
	}

	pub fn set_nlookup(&mut self, nlookup: u64) {
		self.nlookup = nlookup;
	}

	pub fn print(&self) {
		info!("FuseForgetIn: nlookup: {:?}", self.nlookup);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.nlookup.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseForgetOne { // 16 bytes
	nodeid: u64,
	nlookup: u64,
}

impl FuseForgetOne {
	pub fn new(nodeid: u64, nlookup: u64) -> Self {
		Self {
			nodeid,
			nlookup,
		}
	}

	pub fn set_nodeid(&mut self, nodeid: u64) {
		self.nodeid = nodeid;
	}

	pub fn set_nlookup(&mut self, nlookup: u64) {
		self.nlookup = nlookup;
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			nodeid: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
			nlookup: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
		}
	}

	pub fn print(&self) {
		info!("FuseForgetOne: nodeid: {:?}, nlookup: {:?}", self.nodeid, self.nlookup);
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseBatchForgetIn { // 8 bytes
	count: u32,
	dummy: u32,
}

impl FuseBatchForgetIn {
	pub fn new(count: u32) -> Self {
		Self {
			count,
			dummy: 0,
		}
	}

	pub fn set_count(&mut self, count: u32) {
		self.count = count;
	}

	pub fn print(&self) {
		info!("FuseBatchForgetIn: count: {:?}, dummy: {:?}", self.count, self.dummy);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.count.to_le_bytes());
		buf[4..8].copy_from_slice(&self.dummy.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseMknodIn { // 16 bytes
	mode: u32,
	rdev: u32,
	umask: u32,
	padding: u32,
}

impl FuseMknodIn {
	pub fn new(mode: u32, rdev: u32, umask: u32) -> Self {
		Self {
			mode,
			rdev,
			umask,
			padding: 0,
		}
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.mode = mode;
	}

	pub fn set_rdev(&mut self, rdev: u32) {
		self.rdev = rdev;
	}

	pub fn set_umask(&mut self, umask: u32) {
		self.umask = umask;
	}

	pub fn print(&self) {
		info!("FuseMknodIn: mode: {:#x}, rdev: {:#x}, umask: {:?}, padding: {:?}", self.mode, self.rdev, self.umask, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.mode.to_le_bytes());
		buf[4..8].copy_from_slice(&self.rdev.to_le_bytes());
		buf[8..12].copy_from_slice(&self.umask.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseMkdirIn { // 8 bytes
	mode: u32,
	umask: u32,
}

impl FuseMkdirIn {
	pub fn new(mode: u32, umask: u32) -> Self {
		Self {
			mode,
			umask,
		}
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.mode = mode;
	}

	pub fn set_umask(&mut self, umask: u32) {
		self.umask = umask;
	}

	pub fn print(&self) {
		info!("FuseMkdirIn: mode: {:#x}, umask: {:?}", self.mode, self.umask);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.mode.to_le_bytes());
		buf[4..8].copy_from_slice(&self.umask.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseRenameIn { // 8 bytes
	newdir: u64,
}

impl FuseRenameIn {
	pub fn new(newdir: u64) -> Self {
		Self {
			newdir,
		}
	}

	pub fn set_newdir(&mut self, newdir: u64) {
		self.newdir = newdir;
	}

	pub fn print(&self) {
		info!("FuseRenameIn: newdir: {:?}", self.newdir);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.newdir.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseRename2In { // 16 bytes
	newdir: u64,
	flags: u32,
	padding: u32,
}

impl FuseRename2In {
	pub fn new(newdir: u64, flags: u32) -> Self {
		Self {
			newdir,
			flags,
			padding: 0,
		}
	}

	pub fn set_newdir(&mut self, newdir: u64) {
		self.newdir = newdir;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn print(&self) {
		info!("FuseRename2In: newdir: {:?}, flags: {:#x}, padding: {:?}", self.newdir, self.flags, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.newdir.to_le_bytes());
		buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseLinkIn { // 8 bytes
	oldnodeid: u64,
}

impl FuseLinkIn {
	pub fn new(oldnodeid: u64) -> Self {
		Self {
			oldnodeid,
		}
	}

	pub fn set_oldnodeid(&mut self, oldnodeid: u64) {
		self.oldnodeid = oldnodeid;
	}

	pub fn print(&self) {
		info!("FuseLinkIn: oldnodeid: {:?}", self.oldnodeid);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.oldnodeid.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseSetxattrIn { // 16 bytes
	size: u32,
	flags: u32,
	setxattr_flags: u32,
	padding: u32,
}

impl FuseSetxattrIn {
	pub fn new(size: u32, flags: u32, setxattr_flags: u32) -> Self {
		Self {
			size,
			flags,
			setxattr_flags,
			padding: 0,
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.size = size;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_setxattr_flags(&mut self, setxattr_flags: u32) {
		self.setxattr_flags = setxattr_flags;
	}

	pub fn print(&self) {
		info!("FuseSetxattrIn: size: {:?}, flags: {:#x}, setxattr_flags: {:#x}, padding: {:?}", self.size, self.flags, self.setxattr_flags, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.size.to_le_bytes());
		buf[4..8].copy_from_slice(&self.flags.to_le_bytes());
		buf[8..12].copy_from_slice(&self.setxattr_flags.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseGetxattrIn { // 8 bytes
	size: u32,
	padding: u32,
}

impl FuseGetxattrIn {
	pub fn new(size: u32) -> Self {
		Self {
			size,
			padding: 0,
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.size = size;
	}

	pub fn print(&self) {
		info!("FuseGetxattrIn: size: {:?}, padding: {:?}", self.size, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.size.to_le_bytes());
		buf[4..8].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseGetxattrOut { // 8 bytes
	size: u32,
	padding: u32,
}

impl FuseGetxattrOut {
	pub fn new(size: u32) -> Self {
		Self {
			size,
			padding: 0,
		}
	}

	pub fn get_size(&self) -> u32 {
		self.size
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			size: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
			padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
		}
	}

	pub fn print(&self) {
		info!("FuseGetxattrOut: size: {:?}, padding: {:?}", self.size, self.padding);
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseAccessIn { // 8 bytes
	mask: u32,
	padding: u32,
}

impl FuseAccessIn {
	pub fn new(mask: u32) -> Self {
		Self {
			mask,
			padding: 0,
		}
	}

	pub fn set_mask(&mut self, mask: u32) {
		self.mask = mask;
	}

	pub fn print(&self) {
		info!("FuseAccessIn: mask: {:?}, padding: {:?}", self.mask, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..4].copy_from_slice(&self.mask.to_le_bytes());
		buf[4..8].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseFsyncIn { // 8 bytes
	fh: u64,
	fsync_flags: u32,
	padding: u32,
}

impl FuseFsyncIn {
	pub fn new(fh: u64, fsync_flags: u32) -> Self {
		Self {
			fh,
			fsync_flags,
			padding: 0,
		}
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_fsync_flags(&mut self, fsync_flags: u32) {
		self.fsync_flags = fsync_flags;
	}

	pub fn print(&self) {
		info!("FuseFsyncIn: fh: {:#x}, fsync_flags: {:#x}, padding: {:?}", self.fh, self.fsync_flags, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..12].copy_from_slice(&self.fsync_flags.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
	}
}


#[derive(Debug, Clone, Copy)]
pub struct FuseBmapIn { // 16 bytes
	block: u64,
	blocksize: u32,
	padding: u32,
}

impl FuseBmapIn {
	pub fn new(block: u64, blocksize: u32) -> Self {
		Self {
			block,
			blocksize,
			padding: 0,
		}
	}

	pub fn set_block(&mut self, block: u64) {
		self.block = block;
	}

	pub fn set_blocksize(&mut self, blocksize: u32) {
		self.blocksize = blocksize;
	}

	pub fn print(&self) {
		info!("FuseBmapIn: block: {:?}, blocksize: {:?}, padding: {:?}", self.block, self.blocksize, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.block.to_le_bytes());
		buf[8..12].copy_from_slice(&self.blocksize.to_le_bytes());
		buf[12..16].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseBmapOut { // 8 bytes
	block: u64,
}

impl FuseBmapOut {
	pub fn new(block: u64) -> Self {
		Self {
			block,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			block: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
		}
	}

	pub fn get_block(&self) -> u64 {
		self.block
	}

	pub fn print(&self) {
		info!("FuseBmapOut: block: {:?}", self.block);
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlIn { // 32 bytes
	fh: u64,
	flags: u32,
	cmd: u32,
	arg: u64,
	in_size: u32,
	out_size: u32,
}

impl FuseIoctlIn {
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

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_cmd(&mut self, cmd: u32) {
		self.cmd = cmd;
	}

	pub fn set_arg(&mut self, arg: u64) {
		self.arg = arg;
	}

	pub fn set_in_size(&mut self, in_size: u32) {
		self.in_size = in_size;
	}

	pub fn set_out_size(&mut self, out_size: u32) {
		self.out_size = out_size;
	}

	pub fn print(&self) {
		info!("FuseIoctlIn: fh: {:#x}, flags: {:#x}, cmd: {:?}, arg: {:?}, in_size: {:?}, out_size: {:?}", self.fh, self.flags, self.cmd, self.arg, self.in_size, self.out_size);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..12].copy_from_slice(&self.flags.to_le_bytes());
		buf[12..16].copy_from_slice(&self.cmd.to_le_bytes());
		buf[16..24].copy_from_slice(&self.arg.to_le_bytes());
		buf[24..28].copy_from_slice(&self.in_size.to_le_bytes());
		buf[28..32].copy_from_slice(&self.out_size.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlIovec { // 16 bytes
	base: u64,
	len: u64,
}

impl FuseIoctlIovec {
	pub fn new(base: u64, len: u64) -> Self {
		Self {
			base,
			len,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			base: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
			len: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
		}
	}

	pub fn get_base(&self) -> u64 {
		self.base
	}

	pub fn get_len(&self) -> u64 {
		self.len
	}

	pub fn print(&self) {
		info!("FuseIoctlIovec: base: {:?}, len: {:?}", self.base, self.len);
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseIoctlOut { // 16 bytes
	result: i32,
	flags: u32,
	in_iovs: u32,
	out_iovs: u32,
}

impl FuseIoctlOut {
	pub fn new(result: i32, flags: u32, in_iovs: u32, out_iovs: u32) -> Self {
		Self {
			result,
			flags,
			in_iovs,
			out_iovs,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			result: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
			flags: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
			in_iovs: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
			out_iovs: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
		}
	}

	pub fn get_result(&self) -> i32 {
		self.result
	}

	pub fn get_flags(&self) -> u32 {
		self.flags
	}

	pub fn get_in_iovs(&self) -> u32 {
		self.in_iovs
	}

	pub fn get_out_iovs(&self) -> u32 {
		self.out_iovs
	}

	pub fn print(&self) {
		info!("FuseIoctlOut: result: {:?}, flags: {:#x}, in_iovs: {:?}, out_iovs: {:?}", self.result, self.flags, self.in_iovs, self.out_iovs);
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FusePollIn { // 24 bytes
	fh: u64,
	kh: u64,
	flags: u32,
	events: u32,
}

impl FusePollIn {
	pub fn new(fh: u64, kh: u64, flags: u32, events: u32) -> Self {
		Self {
			fh,
			kh,
			flags,
			events,
		}
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_kh(&mut self, kh: u64) {
		self.kh = kh;
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}

	pub fn set_events(&mut self, events: u32) {
		self.events = events;
	}

	pub fn print(&self) {
		info!("FusePollIn: fh: {:#x}, kh: {:?}, flags: {:#x}, events: {:?}", self.fh, self.kh, self.flags, self.events);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..16].copy_from_slice(&self.kh.to_le_bytes());
		buf[16..20].copy_from_slice(&self.flags.to_le_bytes());
		buf[20..24].copy_from_slice(&self.events.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FusePollOut { // 8 bytes
	revents: u32,
	padding: u32,
}

impl FusePollOut {
	pub fn new(revents: u32) -> Self {
		Self {
			revents,
			padding: 0,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			revents: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
			padding: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
		}
	}

	pub fn get_revents(&self) -> u32 {
		self.revents
	}

	pub fn print(&self) {
		info!("FusePollOut: revents: {:?}, padding: {:?}", self.revents, self.padding);
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FuseLseekIn { // 24 bytes
	fh: u64,	
	offset: u64,
	whence: u32,
	padding: u32,
}

impl FuseLseekIn {
	pub fn new(fh: u64, offset: u64, whence: u32) -> Self {
		Self {
			fh,
			offset,
			whence,
			padding: 0,
		}
	}

	pub fn set_fh(&mut self, fh: u64) {
		self.fh = fh;
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.offset = offset;
	}

	pub fn set_whence(&mut self, whence: u32) {
		self.whence = whence;
	}

	pub fn print(&self) {
		info!("FuseLseekIn: fh: {:#x}, offset: {:?}, whence: {:?}, padding: {:?}", self.fh, self.offset, self.whence, self.padding);
	}

	pub fn write_to(&self, buf: &mut [u8]) {
		buf[0..8].copy_from_slice(&self.fh.to_le_bytes());
		buf[8..16].copy_from_slice(&self.offset.to_le_bytes());
		buf[16..20].copy_from_slice(&self.whence.to_le_bytes());
		buf[20..24].copy_from_slice(&self.padding.to_le_bytes());
	}
	
}

#[derive(Debug, Clone, Copy)]
pub struct FuseLseekOut { // 8 bytes
	offset: u64,
}

impl FuseLseekOut {
	pub fn new(offset: u64) -> Self {
		Self {
			offset,
		}
	}

	pub fn read_from(buf: &[u8]) -> Self {
		Self {
			offset: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
		}
	}

	pub fn get_offset(&self) -> u64 {
		self.offset
	}

	pub fn print(&self) {
		info!("FuseLseekOut: offset: {:?}", self.offset);
	}
	
}

#[derive(Debug)]
pub struct FuseDirent { // 24 bytes + len
	ino: u64,
	off: u64,
	namelen: u32,
	type_: u32,
	name: String,
}

impl FuseDirent {
	pub fn new(ino: u64, off: u64, namelen: u32, type_: u32, name: String) -> Self {
		Self {
			ino,
			off,
			namelen,
			type_,
			name,
		}
	}

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

	pub fn get_ino(&self) -> u64 {
		self.ino
	}

	pub fn get_off(&self) -> u64 {
		self.off
	}

	pub fn get_namelen(&self) -> u32 {
		self.namelen
	}

	pub fn get_type(&self) -> u32 {
		self.type_
	}

	pub fn get_name(&self) -> String {
		self.name.clone()
	}
	
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

	pub fn get_len(&self) -> usize {
		let padding = (8 - (self.namelen % 8)) % 8;
		info!("FuseDirent: padding: {:?}, len: {:?}", padding, (self.namelen + padding) as usize);
        24 + (self.namelen + padding) as usize
	}

	pub fn print(&self) {
		info!("FuseDirent: ino: {:?}, off: {:?}, namelen: {:?}, type: {:?}, name: {:?}", self.ino, self.off, self.namelen, self.type_, self.name);
	}
	
}