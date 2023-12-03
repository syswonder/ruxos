/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::Config;

pub fn build_cargo_tree_cmd(cfg: &Config) -> String {
    let default_opt = match cfg.no_default {
        true => "",
        false => "--no-default-features"
    };

    let features_opt = match cfg.features.len() {
        0 => "".to_string(),
        _ => "-F ".to_string() + cfg.features.join(" ").as_str()
    };
    let path = &cfg.loc;
    let cmd_str = format!(
        "cd {path} && cargo tree -e normal,build {default_opt} {features_opt} --format {{p}} --prefix depth",
    );
    cmd_str.to_string()
}
