/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
#![allow(clippy::identity_op)]
#![allow(dead_code)]

use alloc::{string::String, string::ToString, sync::Arc, vec, vec::Vec};
use log::*;
use ruxdriver::prelude::*;
use spin::RwLock;

const EIO: u8 = 5;
const EINVAL: u8 = 90;
const _9P_LEAST_QLEN: u32 = 7; // size[4] type_id[1] tag[2]
pub const _9P_MAX_QSIZE: u32 = 8192 + 1; // it should larger than 8192, or virtio-9p may raise a warning.
pub const _9P_MAX_PSIZE: u32 = 8192 + 1; // it should larger than 8192, or virtio-9p may raise a warning.
const _9P_NONUNAME: u32 = 0;

pub const _9P_SETATTR_MODE: u64 = 0x00000001;
pub const _9P_SETATTR_UID: u64 = 0x00000002;
pub const _9P_SETATTR_GID: u64 = 0x00000004;
pub const _9P_SETATTR_SIZE: u64 = 0x00000008;
pub const _9P_SETATTR_ATIME: u64 = 0x00000010;
pub const _9P_SETATTR_MTIME: u64 = 0x00000020;
pub const _9P_SETATTR_CTIME: u64 = 0x00000040;
pub const _9P_SETATTR_ATIME_SET: u64 = 0x00000080;
pub const _9P_SETATTR_MTIME_SET: u64 = 0x00000100;

const FID_MAX: u32 = 4096;

pub struct Drv9pOps {
    transport: Arc<RwLock<Ax9pDevice>>,
    fid_gen: RwLock<Vec<u32>>,
}

impl Drv9pOps {
    pub fn new(transport: Ax9pDevice) -> Self {
        match transport.init() {
            Ok(_) => {
                info!("9p dev init success");
            }
            Err(ecode) => {
                error!("9p dev init fail! error code:{}", ecode);
            }
        }
        Self {
            transport: Arc::new(RwLock::new(transport)),
            fid_gen: RwLock::new((0..=FID_MAX).rev().collect::<Vec<u32>>()),
        }
    }

    // Send request and receive response
    pub fn request(&mut self, request: &[u8], response: &mut [u8]) -> Result<(), u8> {
        if request.len() as u32 > _9P_MAX_QSIZE {
            return Err(EINVAL);
        }
        let enqueue_try = self.transport.write().send_with_recv(request, response);
        match enqueue_try {
            Ok(_) => {
                const RTYPE_INDEX: usize = 4;
                const ECODE_INDEX: usize = 7;
                const ERROR_RESP: u8 = _9PType::Rlerror as u8;
                match response[RTYPE_INDEX] {
                    ERROR_RESP => {
                        debug!(
                            "9pfs request({}) occurs a error, errcode: {}",
                            request[RTYPE_INDEX], response[ECODE_INDEX]
                        );
                        Err(response[ECODE_INDEX])
                    }
                    _ => Ok(()),
                }
            }
            Err(_) => Err(EIO),
        }
    }

    /// get a new unique fid from uid pool.
    pub fn get_fid(&mut self) -> Option<u32> {
        let mut fid_gen = self.fid_gen.write();
        fid_gen.pop()
    }

    /// recycle a fid for the use of next fops.
    pub fn recycle_fid(&mut self, id: u32) {
        let mut fid_gen = self.fid_gen.write();
        if fid_gen.contains(&id) {
            warn!("fid {} already exist", id);
        } else {
            fid_gen.push(id);
        }
    }

    /// The Terror implement in 9P2000.L, Terror is usually not needed(So it is not implement).
    pub fn l_terror(&mut self, ecode: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tlerror);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(ecode);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tversion(&mut self, protocol: &str) -> Result<String, u8> {
        let mut request = _9PReq::new(_9PType::Tversion);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(_9P_MAX_QSIZE);
        // TODO: use version selected by configure
        request.write_str(protocol);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => {
                const START: usize = 13;
                let length: usize =
                    (response_buffer[12] as usize * 256) + response_buffer[11] as usize;
                Ok(String::from_utf8_lossy(&response_buffer[START..START + length]).to_string())
            }
            Err(err_code) => Err(err_code),
        }
    }

    fn tflush(&mut self, oldtag: u16) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tflush);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u16(oldtag);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tfsync(&mut self, fid: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tfsync);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tclunk(&mut self, fid: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tclunk);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tauth(&mut self, afid: u32, uname: &str, aname: &str) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tauth);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(afid);
        request.write_str(uname);
        request.write_str(aname);
        request.write_u32(_9P_NONUNAME);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tattach(&mut self, fid: u32, afid: u32, uname: &str, aname: &str) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tattach);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(afid);
        request.write_str(uname);
        request.write_str(aname);
        request.write_u32(_9P_NONUNAME);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// `twalk()`: Pay attention to the max_size of request buffer, wnames should not be too long usually.
    pub fn twalk(&mut self, fid: u32, newfid: u32, nwname: u16, wnames: &[&str]) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Twalk);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(newfid);
        request.write_u16(nwname);
        for s in wnames {
            request.write_str(s);
        }
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tcreate(&mut self, fid: u32, name: &str, perm: u32, mode: u8) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tcreate);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_str(name);
        request.write_u32(perm);
        request.write_u8(mode);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn u_tcreate(
        &mut self,
        fid: u32,
        name: &str,
        perm: u32,
        mode: u8,
        extension: &str,
    ) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tcreate);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_str(name);
        request.write_u32(perm);
        request.write_u8(mode);
        request.write_str(extension);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// lcreate creates a regular file name in directory fid and prepares it for I/O.
    pub fn l_tcreate(
        &mut self,
        fid: u32,
        name: &str,
        flags: u32,
        mode: u32,
        gid: u32,
    ) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tlcreate);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_str(name);
        request.write_u32(flags);
        request.write_u32(mode);
        request.write_u32(gid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// This operation will eventually be replaced by renameat (see below).
    pub fn trename(&mut self, fid: u32, dfid: u32, new_name: &str) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::TrenameAt);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(dfid);
        request.write_str(new_name);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// Change the name of a file from oldname to newname,
    /// possible moving it from old directory represented by olddirfid to new directory represented by newdirfid.
    /// If the server returns ENOTSUPP, the client should fall back to the rename operation.
    pub fn trename_at(
        &mut self,
        olddirfid: u32,
        oldname: &str,
        newdirfid: u32,
        new_name: &str,
    ) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::TrenameAt);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(olddirfid);
        request.write_str(oldname);
        request.write_u32(newdirfid);
        request.write_str(new_name);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// open file or dir in 9P2000(.U) Operation
    pub fn topen(&mut self, fid: u32, mode: u8) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Topen);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u8(mode);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// open file or dir in 9P2000.L Operation
    pub fn l_topen(&mut self, fid: u32, flags: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tlopen);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(flags);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// read perform I/O on the file represented by fid.
    /// Note that in v9fs, a read(2) or write(2) system call for a chunk of the file that won't fit in a single request is broken up into multiple requests.
    /// `tread()` can only read _9P_MAX_PSIZE-25 bytes if counter is larger than _9P_MAX_PSIZE.
    pub fn tread(&mut self, fid: u32, offset: u64, count: u32) -> Result<Vec<u8>, u8> {
        // check if `count` larger than MAX_READ_LEN
        const MAX_READ_LEN: u32 = _9P_MAX_PSIZE - 32;
        let mut reading_len = count;
        if reading_len > MAX_READ_LEN {
            reading_len = MAX_READ_LEN;
        }
        let mut request = _9PReq::new(_9PType::Tread);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u64(offset);
        request.write_u32(reading_len);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => {
                const COUNT_START: usize = 7;
                const COUNT_END: usize = 11;
                let length = lbytes2u64(&response_buffer[COUNT_START..COUNT_START + 4]) as usize;
                Ok(response_buffer[COUNT_END..COUNT_END + length].to_vec())
            }
            Err(err_code) => Err(err_code),
        }
    }

    pub fn treaddir(&mut self, fid: u32) -> Result<Vec<DirEntry>, u8> {
        let mut dir_entries: Vec<DirEntry> = Vec::new();
        let mut offptr = 0_u64;
        loop {
            // check if `count` larger than MAX_READ_LEN
            const MAX_READ_LEN: u32 = _9P_MAX_PSIZE - 32;

            let mut request = _9PReq::new(_9PType::Treaddir);
            let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
            request.write_u32(fid);
            request.write_u64(offptr);
            request.write_u32(MAX_READ_LEN);
            request.finish();
            match self.request(&request.buffer, &mut response_buffer) {
                Ok(_) => {
                    const COUNT_START: usize = 7;
                    const COUNT_END: usize = 11;
                    let length: u64 = lbytes2u64(&response_buffer[COUNT_START..COUNT_START + 4]);
                    if length == 0 {
                        break;
                    }

                    let mut resp_ptr = COUNT_END;
                    while resp_ptr < length as usize {
                        // qid[13] offset[8] type[1] name[s]
                        let dir_entry = DirEntry {
                            qid: _9PQid::new(&response_buffer[resp_ptr..resp_ptr + 13]),
                            offset: lbytes2u64(&response_buffer[resp_ptr + 13..resp_ptr + 21]),
                            dtype: response_buffer[resp_ptr + 21],
                            name: lbytes2str(&response_buffer[resp_ptr + 22..]),
                        };
                        resp_ptr += 24 + dir_entry.name.len();
                        offptr = dir_entry.offset;
                        dir_entries.push(dir_entry);
                    }
                }
                Err(err_code) => return Err(err_code),
            }
        }
        Ok(dir_entries)
    }

    /// read directory represented by fid in 9P2000.u. In 9P2000.L, using treaddir() instead.
    pub fn u_treaddir(&mut self, fid: u32) -> Result<Vec<DirEntry>, u8> {
        let mut dir_entries: Vec<DirEntry> = Vec::new();
        let mut offptr = 0_u64;
        loop {
            // check if `count` larger than MAX_READ_LEN
            const MAX_READ_LEN: u32 = _9P_MAX_PSIZE - 32;

            let mut request = _9PReq::new(_9PType::Tread);
            let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
            request.write_u32(fid);
            request.write_u64(offptr);
            request.write_u32(MAX_READ_LEN);
            request.finish();
            match self.request(&request.buffer, &mut response_buffer) {
                Ok(_) => {
                    const COUNT_START: usize = 7;
                    const COUNT_END: usize = 11;
                    let length: u64 = lbytes2u64(&response_buffer[COUNT_START..COUNT_START + 4]);
                    if length == 0 {
                        break;
                    }

                    let mut resp_ptr = COUNT_END;
                    while resp_ptr < length as usize {
                        // qid[13] offset[8] type[1] name[s]
                        let state = UStatFs::parse_u_from(&response_buffer[resp_ptr..]);
                        let dir_entry = DirEntry {
                            qid: state.get_qid(),
                            offset: resp_ptr as u64,
                            dtype: state.get_ftype(),
                            name: state.get_name(),
                        };
                        resp_ptr += state.get_self_length();
                        offptr = dir_entry.offset;
                        dir_entries.push(dir_entry);
                    }
                }
                Err(err_code) => return Err(err_code),
            }
        }
        Ok(dir_entries)
    }

    /// write perform I/O on the file represented by fid.
    /// Note that in v9fs, a read(2) or write(2) system call for a chunk of the file that won't fit in a single request is broken up into multiple requests.
    pub fn twrite(&mut self, fid: u32, offset: u64, data: &[u8]) -> Result<usize, u8> {
        const MAX_READ_LEN: u32 = _9P_MAX_PSIZE - 32;
        let mut writing_len = data.len() as u32;
        if writing_len > MAX_READ_LEN {
            writing_len = MAX_READ_LEN;
        }
        let mut request = _9PReq::new(_9PType::Twrite);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u64(offset);
        request.write_u32(writing_len);
        for value in &data[..writing_len as usize] {
            request.write_u8(*value);
        }
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok(lbytes2u64(&response_buffer[7..11]) as usize), // index from 7 to 11 corresponing to total count of writed byte
            Err(ecode) => Err(ecode),
        }
    }

    pub fn tmkdir(&mut self, dfid: u32, name: &str, mode: u32, gid: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tmkdir);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(dfid);
        request.write_str(name);
        request.write_u32(mode);
        request.write_u32(gid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tremove(&mut self, fid: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tremove);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// size[4] Rgetattr tag[2] valid[8] qid[13] mode[4] uid[4] gid[4] nlink[8] rdev[8] size[8] blksize[8] blocks[8] atime_sec[8]
    /// atime_nsec[8] mtime_sec[8] mtime_nsec[8] ctime_sec[8] ctime_nsec[8] btime_sec[8] btime_nsec[8] gen[8] data_version[8]
    pub fn tgetattr(&mut self, fid: u32, request_mask: u64) -> Result<FileAttr, u8> {
        let mut request = _9PReq::new(_9PType::Tgetattr);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u64(request_mask);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok(FileAttr {
                vaild: lbytes2u64(&response_buffer[7..15]),
                qid: _9PQid::new(&response_buffer[15..28]),
                mode: lbytes2u64(&response_buffer[28..32]) as u32,
                uid: lbytes2u64(&response_buffer[32..36]) as u32,
                gid: lbytes2u64(&response_buffer[36..40]) as u32,
                n_link: lbytes2u64(&response_buffer[40..48]),
                rdev: lbytes2u64(&response_buffer[48..56]),
                size: lbytes2u64(&response_buffer[56..64]),
                blk_size: lbytes2u64(&response_buffer[64..72]),
                n_blk: lbytes2u64(&response_buffer[72..80]),
                atime_sec: lbytes2u64(&response_buffer[80..88]),
                atime_ns: lbytes2u64(&response_buffer[88..96]),
                mtime_sec: lbytes2u64(&response_buffer[96..104]),
                mtime_ns: lbytes2u64(&response_buffer[104..112]),
                ctime_sec: lbytes2u64(&response_buffer[112..120]),
                ctime_ns: lbytes2u64(&response_buffer[120..128]),
                btime_sec: lbytes2u64(&response_buffer[128..136]),
                btime_ns: lbytes2u64(&response_buffer[136..144]),
                gen: lbytes2u64(&response_buffer[144..152]),
                date_version: lbytes2u64(&response_buffer[152..160]),
            }),
            Err(err_code) => Err(err_code),
        }
    }

    pub fn tsetattr(&mut self, fid: u32, attr: FileAttr) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tsetattr);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(attr.vaild as u32);
        request.write_u32(attr.mode);
        request.write_u32(attr.uid);
        request.write_u32(attr.gid);
        request.write_u64(attr.size);
        request.write_u64(attr.atime_sec);
        request.write_u64(attr.atime_ns);
        request.write_u64(attr.mtime_sec);
        request.write_u64(attr.mtime_ns);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// xattrwalk gets a newfid pointing to xattr name. This fid can later be used to read the xattr value.
    /// If name is NULL newfid can be used to get the list of extended attributes associated with the file system object.
    pub fn t_xattr_walk(&mut self, fid: u32, new_fid: u32, name: &str) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::TxattrWalk);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u32(new_fid);
        request.write_str(name);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn t_xattr_create(
        &mut self,
        fid: u32,
        name: &str,
        attr_size: u64,
        flags: u32,
    ) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::TxattrCreate);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_str(name);
        request.write_u64(attr_size);
        request.write_u32(flags);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tsymlink(&mut self, fid: u32, name: &str, symtgt: &str, gid: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tsymlink);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_str(name);
        request.write_str(symtgt);
        request.write_u32(gid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tmknod(
        &mut self,
        dfid: u32,
        name: &str,
        mode: u32,
        major: u32,
        minor: u32,
        gid: u32,
    ) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tmknod);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(dfid);
        request.write_str(name);
        request.write_u32(mode);
        request.write_u32(major);
        request.write_u32(minor);
        request.write_u32(gid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn treadlink(&mut self, fid: u32) -> Result<String, u8> {
        let mut request = _9PReq::new(_9PType::Treadlink);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok(lbytes2str(&response_buffer[7..])),
            Err(err_code) => Err(err_code),
        }
    }

    pub fn tlink(&mut self, dfid: u32, fid: u32, name: &str) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Tlink);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(dfid);
        request.write_u32(fid);
        request.write_str(name);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    pub fn tunlink(&mut self, dirfid: u32, name: &str, flags: u32) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::TunlinkAT);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(dirfid);
        request.write_str(name);
        request.write_u32(flags);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }

    /// create or delete a lock on a fid, similar to fcntl(F_SETLK)
    /// bits of flags: BLOCK 1, RESERVED 1<<1;
    /// return status if ok: SUCCESS 0; BLOCKED 1; ERROR 2; GRACE 3.  
    pub fn tlock(&mut self, fid: u32, flags: u32, locker: PosLock) -> Result<u8, u8> {
        let mut request = _9PReq::new(_9PType::Tlock);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u8(locker.lock_type);
        request.write_u32(flags);
        request.write_u64(locker.start);
        request.write_u64(locker.length);
        request.write_u32(locker.proc_id);
        request.write_str(&locker.client_id);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok(response_buffer[7]),
            Err(ecode) => Err(ecode),
        }
    }

    /// check if lock existing.
    pub fn tgetlock(&mut self, fid: u32, locker: PosLock) -> Result<PosLock, u8> {
        let mut request = _9PReq::new(_9PType::Tgetlock);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.write_u8(locker.lock_type);
        request.write_u64(locker.start);
        request.write_u64(locker.length);
        request.write_u32(locker.proc_id);
        request.write_str(&locker.client_id);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok(PosLock {
                lock_type: response_buffer[7],
                start: lbytes2u64(&response_buffer[8..16]),
                length: lbytes2u64(&response_buffer[16..24]),
                proc_id: lbytes2u64(&response_buffer[24..28]) as u32,
                client_id: lbytes2str(&response_buffer[28..]),
            }),
            Err(ecode) => Err(ecode),
        }
    }

    /// get information of filesystem (in 9P2000.L protocol)
    pub fn tstatfs(&mut self, fid: u32) -> Result<LStatFs, u8> {
        let mut request = _9PReq::new(_9PType::Tstatfs);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            Ok(_) => Ok({
                LStatFs {
                    fs_type: lbytes2u64(&response_buffer[7..11]) as u32,
                    blk_size: lbytes2u64(&response_buffer[11..15]) as u32,
                    n_blk: lbytes2u64(&response_buffer[15..23]),
                    blk_free: lbytes2u64(&response_buffer[23..31]),
                    blk_avail: lbytes2u64(&response_buffer[31..39]),
                    n_files: lbytes2u64(&response_buffer[39..47]),
                    file_free: lbytes2u64(&response_buffer[47..55]),
                    fs_id: lbytes2u64(&response_buffer[55..63]),
                    len_name: lbytes2u64(&response_buffer[63..67]) as u32,
                }
            }),
            Err(ecode) => Err(ecode),
        }
    }

    pub fn tstat(&mut self, fid: u32) -> Result<UStatFs, u8> {
        let mut request = _9PReq::new(_9PType::Tstat);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        request.write_u32(fid);
        request.finish();
        match self.request(&request.buffer, &mut response_buffer) {
            // Note: Tstat should start stat[n] from index 7 in respone_buffer, but QEMU's start at index 9.
            // see more at: http://ericvh.github.io/9p-rfc/rfc9p2000.u.html
            Ok(_) => Ok(UStatFs::parse_u_from(&response_buffer[9..])),
            Err(ecode) => Err(ecode),
        }
    }

    pub fn twstat(&mut self, fid: u32, stat: UStatFs) -> Result<(), u8> {
        let mut request = _9PReq::new(_9PType::Twstat);
        let mut response_buffer: [u8; _9P_MAX_PSIZE as usize] = [0; _9P_MAX_PSIZE as usize];
        // Note: twstat should start stat[n] from index 9 in respone_buffer, but QEMU's implement start at index 11.
        // see more at:http://ericvh.github.io/9p-rfc/rfc9p2000.u.html
        request.write_u16(0);
        request.write_u32(fid);
        request.write_u16(stat.size);
        request.write_u16(stat.ktype);
        request.write_u32(stat.dev);
        request.write_u8(stat.qid.ftype);
        request.write_u32(stat.qid.version);
        request.write_u64(stat.qid.path);
        request.write_u32(stat.mode);
        request.write_u32(stat.atime);
        request.write_u32(stat.mtime);
        request.write_u64(stat.length);
        request.write_str(&stat.name);
        request.write_str(&stat.uid);
        request.write_str(&stat.gid);
        request.write_str(&stat.muid);
        request.write_str(&stat.extension);
        request.write_u32(stat.n_uid);
        request.write_u32(stat.n_gid);
        request.write_u32(stat.n_muid);
        request.finish();
        self.request(&request.buffer, &mut response_buffer)
    }
}

fn lbytes2u64(bytes: &[u8]) -> u64 {
    let mut ret: u64 = 0;
    for n in bytes.iter().rev() {
        ret = (ret << 8) + *n as u64;
    }
    ret
}

/// format of string in 9p: length[2] string[length]
fn lbytes2str(bytes: &[u8]) -> String {
    let length = lbytes2u64(&bytes[..2]) as usize;
    let str = String::from_utf8_lossy(&bytes[2..2 + length]);
    str.to_string()
}

struct _9PReq {
    size: u32,
    type_id: u8,
    tag: u16,
    buffer: Vec<u8>, // included size[4] type_id[1] and tag[2]
}

impl _9PReq {
    fn new(qtype: _9PType) -> Self {
        Self {
            size: _9P_LEAST_QLEN,
            type_id: qtype as u8,
            tag: 0,
            buffer: vec![0; 7],
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    fn write_u16(&mut self, value: u16) {
        self.buffer.push((value & 0xff_u16) as u8);
        self.buffer.push(((value & 0xff00_u16) >> 8) as u8);
    }

    fn write_u32(&mut self, value: u32) {
        const U32_SIZE: u32 = 4;
        let mut value = value;
        for _ in 0..U32_SIZE {
            let byte: u8 = (value & 0xff_u32) as u8;
            value >>= 8;
            self.buffer.push(byte);
        }
    }

    fn write_u64(&mut self, value: u64) {
        const U64_SIZE: u32 = 8;
        let mut value = value;
        for _ in 0..U64_SIZE {
            let byte: u8 = (value & 0xff_u64) as u8;
            value >>= 8;
            self.buffer.push(byte);
        }
    }

    fn write_str(&mut self, value: &str) {
        let str_size: u32 = value.as_bytes().len() as u32;
        self.write_u16(str_size as u16);
        for cbyte in value.bytes() {
            self.buffer.push(cbyte);
        }
    }

    fn finish(&mut self) {
        const U32_SIZE: u32 = 4;
        const TYPE_IDNEX: usize = 4;
        const TAG_IDNEX: usize = 5;
        let mut value = self.buffer.len() as u32;
        for i in 0..U32_SIZE {
            let byte = (value & 0xff_u32) as u8;
            value >>= 8;
            self.buffer[i as usize] = byte;
        }
        self.buffer[TYPE_IDNEX] = self.type_id;
        self.buffer[TAG_IDNEX] = self.tag as u8;
        self.buffer[TAG_IDNEX + 1] = (self.tag >> 8) as u8;
    }
}

#[derive(Debug)]
pub struct _9PQid {
    ftype: u8,
    version: u32,
    path: u64,
}

impl _9PQid {
    fn new(bytes: &[u8]) -> Self {
        Self {
            ftype: bytes[0],
            version: lbytes2u64(&bytes[1..5]) as u32,
            path: lbytes2u64(&bytes[5..13]),
        }
    }
}

pub struct PosLock {
    lock_type: u8,
    start: u64,
    length: u64,
    proc_id: u32,
    client_id: String,
}

impl PosLock {
    ///for lock_type: RDLCK 0, WRLCK 1, UNLCK 2    
    pub fn new(lock_type: u8, start: u64, length: u64, proc_id: u32, client_id: &str) -> Self {
        Self {
            lock_type,
            start,
            length,
            proc_id,
            client_id: client_id.to_string(),
        }
    }
}

pub struct DirEntry {
    qid: _9PQid,
    offset: u64,
    dtype: u8,
    name: String,
}

impl DirEntry {
    pub fn get_type(&self) -> u8 {
        self.dtype
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

#[derive(Debug)]
pub struct FileAttr {
    vaild: u64, // vaild mask
    qid: _9PQid,
    mode: u32,      // mode for protection
    n_link: u64,    // number of hard links
    uid: u32,       // user ID of owner
    gid: u32,       // group ID of owner
    rdev: u64,      // device ID (if special file)
    size: u64,      // total size, in bytes
    blk_size: u64,  // blocksize for file system I/O
    n_blk: u64,     // number of 512B blocks allocated
    atime_sec: u64, // time of last access
    atime_ns: u64,
    mtime_sec: u64, // time of last modification
    mtime_ns: u64,
    ctime_sec: u64, // time of last status change
    ctime_ns: u64,
    btime_sec: u64, // time of last create change
    btime_ns: u64,
    gen: u64,
    date_version: u64,
}

impl FileAttr {
    // TODO: add function to get/set more attributes
    pub fn new() -> Self {
        Self {
            vaild: 0,
            qid: _9PQid {
                ftype: 0,
                version: 0,
                path: 0,
            },
            mode: 0,      // mode for protection
            n_link: 0,    // number of hard links
            uid: 0,       // user ID of owner
            gid: 0,       // group ID of owner
            rdev: 0,      // device ID (if special file)
            size: 0,      // total size, in bytes
            blk_size: 0,  // blocksize for file system I/O
            n_blk: 0,     // number of 512B blocks allocated
            atime_sec: 0, // time of last access
            atime_ns: 0,
            mtime_sec: 0, // time of last modification
            mtime_ns: 0,
            ctime_sec: 0, // time of last status change
            ctime_ns: 0,
            btime_sec: 0, // time of last create change
            btime_ns: 0,
            gen: 0,
            date_version: 0,
        }
    }

    pub fn get_ftype(&self) -> u8 {
        match self.qid.ftype {
            0x00 => 0o10,
            0x02 => 0o12,
            0x20 => 0o6,
            0x40 => 0o1,
            0x80 => 0o4,
            _ => {
                error!("Unsupported, looking it as File(0o10)!");
                0o10
            }
        }
    }

    pub fn get_perm(&self) -> u32 {
        self.mode
    }

    pub fn set_size(&mut self, size: u64) {
        self.vaild |= _9P_SETATTR_SIZE;
        self.size = size;
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_blk_num(&self) -> u64 {
        self.n_blk
    }
}

pub struct LStatFs {
    fs_type: u32,   /* type of file system (see below) */
    blk_size: u32,  /* optimal transfer block size */
    n_blk: u64,     /* total data blocks in file system */
    blk_free: u64,  /* free blocks in fs */
    blk_avail: u64, /* free blocks avail to non-superuser */
    n_files: u64,   /* total file nodes in file system */
    file_free: u64, /* free file nodes in fs */
    fs_id: u64,     /* file system id */
    len_name: u32,  /* maximum length of filenames */
}

pub struct UStatFs {
    size: u16,
    ktype: u16,
    dev: u32,
    qid: _9PQid,
    mode: u32,
    atime: u32,
    mtime: u32,
    length: u64,
    name: String,
    uid: String,
    gid: String,
    muid: String,
    extension: String,
    n_uid: u32,
    n_gid: u32,
    n_muid: u32,
}

impl UStatFs {
    pub fn new() -> Self {
        Self {
            size: 0,
            ktype: 0,
            dev: 0,
            qid: _9PQid {
                ftype: 0,
                version: 0,
                path: 0,
            },
            mode: 0,
            atime: 0,
            mtime: 0,
            length: 0,
            name: "".to_string(),
            uid: "".to_string(),
            gid: "".to_string(),
            muid: "".to_string(),
            extension: "".to_string(),
            n_uid: 0,
            n_gid: 0,
            n_muid: 0,
        }
    }

    pub fn parse_u_from(bytes: &[u8]) -> Self {
        let name_idx = 41_usize;
        let uid_idx = name_idx + (lbytes2u64(&bytes[name_idx..name_idx + 2]) as usize + 2);
        let gid_idx = uid_idx + (lbytes2u64(&bytes[uid_idx..uid_idx + 2]) as usize + 2);
        let muid_idx = gid_idx + (lbytes2u64(&bytes[gid_idx..gid_idx + 2]) as usize + 2);
        let extension_idx = muid_idx + (lbytes2u64(&bytes[muid_idx..muid_idx + 2]) as usize + 2);
        let extension_end =
            extension_idx + (lbytes2u64(&bytes[extension_idx..extension_idx + 2]) as usize + 2);
        Self {
            size: lbytes2u64(&bytes[0..2]) as u16,
            ktype: lbytes2u64(&bytes[2..4]) as u16,
            dev: lbytes2u64(&bytes[4..8]) as u32,
            qid: _9PQid::new(&bytes[8..21]),
            mode: lbytes2u64(&bytes[21..25]) as u32,
            atime: lbytes2u64(&bytes[25..29]) as u32,
            mtime: lbytes2u64(&bytes[29..33]) as u32,
            length: lbytes2u64(&bytes[33..41]),
            name: lbytes2str(&bytes[name_idx..uid_idx]),
            uid: lbytes2str(&bytes[uid_idx..gid_idx]),
            gid: lbytes2str(&bytes[gid_idx..muid_idx]),
            muid: lbytes2str(&bytes[muid_idx..extension_idx]),
            extension: lbytes2str(&bytes[extension_idx..extension_end]),
            n_uid: lbytes2u64(&bytes[extension_end..extension_end + 4]) as u32,
            n_gid: lbytes2u64(&bytes[extension_end + 4..extension_end + 8]) as u32,
            n_muid: lbytes2u64(&bytes[extension_end + 8..extension_end + 12]) as u32,
        }
    }

    pub fn set_length(&mut self, length: u64) {
        self.length = length;
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub fn get_blk_num(&self) -> u64 {
        0
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_perm(&self) -> u32 {
        self.mode
    }

    pub fn get_ftype(&self) -> u8 {
        match self.qid.ftype {
            0x00 => 0o10,
            0x02 => 0o12,
            0x20 => 0o6,
            0x40 => 0o1,
            0x80 => 0o4,
            _ => {
                error!("Unsupported, looking it as File(0o10)!");
                0o10
            }
        }
    }

    pub fn get_qid(&self) -> _9PQid {
        _9PQid {
            ftype: self.qid.ftype,
            version: self.qid.version,
            path: self.qid.path,
        }
    }

    pub fn get_self_length(&self) -> usize {
        self.name.len()
            + self.uid.len()
            + self.gid.len()
            + self.muid.len()
            + self.extension.len()
            + 63
    }
}

enum _9PType {
    Tlerror = 6,
    Rlerror,
    Tstatfs = 8,
    Rstatfs,
    Tlopen = 12,
    Rlopen,
    Tlcreate = 14,
    Rlcreate,
    Tsymlink = 16,
    Rsymlink,
    Tmknod = 18,
    Rmknod,
    Trename = 20,
    Rrename,
    Treadlink = 22,
    Rreadlink,
    Tgetattr = 24,
    Rgetattr,
    Tsetattr = 26,
    Rsetattr,
    TxattrWalk = 30,
    RxattrWalk,
    TxattrCreate = 32,
    RxattrCreate,
    Treaddir = 40,
    Rreaddir,
    Tfsync = 50,
    Rfsync,
    Tlock = 52,
    Rlock,
    Tgetlock = 54,
    Rgetlock,
    Tlink = 70,
    Rlink,
    Tmkdir = 72,
    Rmkdir,
    TrenameAt = 74,
    RrenameAt,
    TunlinkAT = 76,
    RunlinkAT,
    Tversion = 100,
    Rversion,
    Tauth = 102,
    Rauth,
    Tattach = 104,
    Rattach,
    Terror = 106,
    Rerror,
    Tflush = 108,
    Rflush,
    Twalk = 110,
    Rwalk,
    Topen = 112,
    Ropen,
    Tcreate = 114,
    Rcreate,
    Tread = 116,
    Rread,
    Twrite = 118,
    Rwrite,
    Tclunk = 120,
    Rclunk,
    Tremove = 122,
    Rremove,
    Tstat = 124,
    Rstat,
    Twstat = 126,
    Rwstat,
}
