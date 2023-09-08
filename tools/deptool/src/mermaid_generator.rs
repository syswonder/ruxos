/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use std::collections::HashMap;

use crate::{parse_deps, cmd_parser::is_arceos_crate};

pub fn gen_mermaid_script(deps: &String, result: &mut String) {
    let deps_parsed = parse_deps(&deps);
    let dep_root = &deps_parsed[0];

    let mut parsed_crates: Vec<&String> = Vec::new();
    let mut lastest_dep_map: HashMap<i32, &String> = HashMap::new();
    let mut idx: usize = 1;

    lastest_dep_map.insert(0, &dep_root.1);
    while idx < deps_parsed.len() {
        let (level, name) = deps_parsed.get(idx).unwrap();
        if !is_arceos_crate(&name) {
            idx += 1;
            continue;
        }
        *result += &format!("{}-->{}\n", lastest_dep_map[&(level - 1)], name);
        if parsed_crates.contains(&name) {
            let mut skip_idx: usize = idx + 1;
            if skip_idx >= deps_parsed.len() {
                break;
            }
            while deps_parsed.get(skip_idx).unwrap().0 > *level {
                idx += 1;
                skip_idx += 1;
            }
            idx += 1;
        } else {
            parsed_crates.push(&name);
            lastest_dep_map.insert(*level, name);
            idx += 1;
        }
    }
}
