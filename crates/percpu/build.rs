/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use std::path::Path;

fn main() {
    if cfg!(target_os = "linux") && cfg!(not(feature = "sp-naive")) {
        let ld_script_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("test_percpu.x");
        println!("cargo:rustc-link-arg-tests=-no-pie");
        println!("cargo:rustc-link-arg-tests=-T{}", ld_script_path.display());
    }
}
