/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use core::ffi::{c_char, c_int, c_void};
use ruxos_posix_api::{environ, environ_iter, RUX_ENVIRON};

use crate::malloc::{free, malloc};
use crate::string::strlen;
unsafe fn find_env(search: *const c_char) -> Option<(usize, *mut c_char)> {
    for (i, mut item) in environ_iter().enumerate() {
        let mut search = search;
        loop {
            let end_of_query = *search == 0 || *search == b'=' as c_char;
            assert_ne!(*item, 0, "environ has an item without value");
            if *item == b'=' as c_char || end_of_query {
                if *item == b'=' as c_char && end_of_query {
                    // Both keys env here
                    return Some((i, item.add(1)));
                } else {
                    break;
                }
            }

            if *item != *search {
                break;
            }

            item = item.add(1);
            search = search.add(1);
        }
    }
    None
}

unsafe fn put_new_env(insert: *mut c_char) {
    // XXX: Another problem is that `environ` can be set to any pointer, which means there is a
    // chance of a memory leak. But we can check if it was the same as before, like musl does.
    if environ == RUX_ENVIRON.as_mut_ptr() {
        *RUX_ENVIRON.last_mut().unwrap() = insert;
        RUX_ENVIRON.push(core::ptr::null_mut());
        // Likely a no-op but is needed due to Stacked Borrows.
        environ = RUX_ENVIRON.as_mut_ptr();
    } else {
        RUX_ENVIRON.clear();
        RUX_ENVIRON.extend(environ_iter());
        RUX_ENVIRON.push(insert);
        RUX_ENVIRON.push(core::ptr::null_mut());
        environ = RUX_ENVIRON.as_mut_ptr();
    }
}

unsafe fn copy_kv(
    existing: *mut c_char,
    key: *const c_char,
    value: *const c_char,
    key_len: usize,
    value_len: usize,
) {
    core::ptr::copy_nonoverlapping(key, existing, key_len);
    core::ptr::write(existing.add(key_len), b'=' as c_char);
    core::ptr::copy_nonoverlapping(value, existing.add(key_len + 1), value_len);
    core::ptr::write(existing.add(key_len + 1 + value_len), 0);
}

/// set an environ variable
#[no_mangle]
pub unsafe extern "C" fn setenv(
    key: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    let key_len = strlen(key);
    let value_len = strlen(value);
    if let Some((i, existing)) = find_env(key) {
        if overwrite == 0 {
            return 0;
        }

        let existing_len = strlen(existing);
        if existing_len >= value_len {
            // Reuse existing element's allocation
            core::ptr::copy_nonoverlapping(value, existing, value_len);
            core::ptr::write(existing.add(value_len), 0);
        } else {
            // Reuse environ slot, but allocate a new pointer.
            let ptr = malloc(key_len + 1 + value_len + 1) as *mut c_char;
            copy_kv(ptr, key, value, key_len, value_len);
            environ.add(i).write(ptr);
        }
    } else {
        // Expand environ and allocate a new pointer.
        let ptr = malloc(key_len + 1 + value_len + 1) as *mut c_char;
        copy_kv(ptr, key, value, key_len, value_len);
        put_new_env(ptr);
    }
    0
}

/// unset an environ variable
#[no_mangle]
pub unsafe extern "C" fn unsetenv(key: *const c_char) -> c_int {
    if let Some((i, _)) = find_env(key) {
        if environ == RUX_ENVIRON.as_mut_ptr() {
            // No need to worry about updating the pointer, this does not
            // reallocate in any way. And the final null is already shifted back.
            let rm = RUX_ENVIRON.remove(i);
            free(rm as *mut c_void);
            // My UB paranoia.
            environ = RUX_ENVIRON.as_mut_ptr();
        } else {
            let len = RUX_ENVIRON.len();
            for _ in 0..len {
                let rm = RUX_ENVIRON.pop().unwrap();
                free(rm as *mut c_void);
            }
            RUX_ENVIRON.extend(
                environ_iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, v)| v),
            );
            RUX_ENVIRON.push(core::ptr::null_mut());
            environ = RUX_ENVIRON.as_mut_ptr();
        }
    }
    0
}

/// get the corresponding environ variable
#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    find_env(name)
        .map(|val| val.1)
        .unwrap_or(core::ptr::null_mut())
}
