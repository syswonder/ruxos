use crate::{ctypes::kstat, utils::char_ptr_to_str, *};
use alloc::{vec, vec::Vec};
use core::{
    ffi::c_char,
    ptr::{null, null_mut},
};

#[derive(Debug)]
pub struct ElfProg {
    pub name: Vec<u8>,
    pub path: Vec<u8>,
    pub platform: Vec<u8>,
    pub rand: Vec<u64>,
    pub base: usize,
    pub entry: usize,
    pub interp_path: *const c_char,
    pub phent: usize,
    pub phnum: usize,
    pub phdr: usize,
}

impl ElfProg {
    /// read elf from `path`, and copy LOAD segments to a alloacated memory
    ///
    /// and load interp, if needed.
    pub fn new(filepath: *const c_char) -> Self {
        let name = char_ptr_to_str(filepath).unwrap().as_bytes().to_vec();
        let path = name.clone();
        debug!("sys_execve: new elf prog: {:?}", char_ptr_to_str(filepath));

        // open file
        let fd = sys_open(filepath, ctypes::O_RDWR as i32, 0);

        // get file size
        let mut buf = ctypes::kstat {
            ..Default::default()
        };
        sys_fstat(fd, &mut buf as *const kstat as *mut _);
        let filesize = buf.st_size as usize;

        // read file
        let mut file = vec![0u8; filesize];
        sys_read(fd, file.as_mut_ptr() as *mut _, filesize);
        debug!("sys_execve: read file size 0x{filesize:x}");
        sys_close(fd);

        // parse elf
        let file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(&file)
            .expect("parse ELF failed");

        // get program's LOAD mem size
        let mut msize = 0;
        let segs = file.segments().unwrap();
        for seg in segs {
            if seg.p_type == elf::abi::PT_LOAD {
                msize += seg.p_memsz;
            }
        }

        // copy LOAD segments
        let base = crate::sys_mmap(null_mut(), msize as usize, 0, 0, 0, 0) as usize;
        for seg in segs {
            if seg.p_type == elf::abi::PT_LOAD {
                let data = file.segment_data(&seg).unwrap();
                let dst = (seg.p_vaddr as usize + base) as *mut u8;
                unsafe { dst.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
            }
        }

        // phdr
        let phdr = base + file.ehdr.e_phoff as usize;

        // get entry
        let entry = file.ehdr.e_entry as usize + base;

        // parse interpreter
        let mut interp_path = null::<c_char>();
        for seg in file.segments().unwrap() {
            if seg.p_type == elf::abi::PT_INTERP {
                let data = file.segment_data(&seg).unwrap();
                interp_path = data.as_ptr() as *const c_char;
                break;
            }
        }

        // platform
        #[cfg(target_arch = "aarch64")]
        let platform = b"aarch64".to_vec();
        #[cfg(target_arch = "x86_64")]
        let platform = b"x86_64".to_vec();
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        let platform = b"unknown".to_vec();

        // get address of .text for debugging
        let text_section_addr = base
            + file
                .section_header_by_name(".text")
                .unwrap()
                .unwrap()
                .sh_offset as usize;
        debug!(
            "sys_execve: loaded ELF in 0x{:x}, .text is 0x{:x}",
            base, text_section_addr
        );

        // create retval
        Self {
            base,
            entry,
            name,
            path,
            platform,
            rand: alloc::vec![1, 2],
            interp_path,
            phent: file.ehdr.e_phentsize as usize,
            phnum: file.ehdr.e_phnum as usize,
            phdr,
        }
    }
}
