/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

const NET_DEV_FEATURES: &[&str] = &["ixgbe", "virtio-net", "loopback"];
const BLOCK_DEV_FEATURES: &[&str] = &["ramdisk", "bcm2835-sdhci", "virtio-blk"];
const DISPLAY_DEV_FEATURES: &[&str] = &["virtio-gpu"];
const _9P_DEV_FEATURES: &[&str] = &["virtio-9p"];

fn has_feature(feature: &str) -> bool {
    let ret = std::env::var(format!(
        "CARGO_FEATURE_{}",
        feature.to_uppercase().replace('-', "_")
    ))
    .is_ok();
    println!(
        "CARGO_FEATURE_{}   {}",
        feature.to_uppercase().replace('-', "_"),
        ret
    );
    ret
}

fn enable_cfg(key: &str, value: &str) {
    println!("cargo:rustc-cfg={key}=\"{value}\"");
}

fn main() {
    if has_feature("bus-pci") {
        enable_cfg("bus", "pci");
    } else {
        enable_cfg("bus", "mmio");
    }

    // Generate cfgs like `net_dev="virtio-net"`. if `dyn` is not enabled, only one device is
    // selected for each device category. If no device is selected, `dummy` is selected.
    let is_dyn = has_feature("dyn");
    for (dev_kind, feat_list) in [
        ("net", NET_DEV_FEATURES),
        ("block", BLOCK_DEV_FEATURES),
        ("display", DISPLAY_DEV_FEATURES),
        ("_9p", _9P_DEV_FEATURES),
    ] {
        if !has_feature(dev_kind) {
            continue;
        }

        let mut selected = false;
        for feat in feat_list {
            if has_feature(feat) {
                enable_cfg(&format!("{dev_kind}_dev"), feat);
                selected = true;
                if !is_dyn {
                    break;
                }
            }
        }
        if !is_dyn && !selected {
            enable_cfg(&format!("{dev_kind}_dev"), "dummy");
        }
    }
}
