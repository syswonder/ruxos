/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::sync::atomic::AtomicUsize;

use alloc::{string::String, sync::Arc, vec, vec::Vec};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

use crate::{driver::TtyDriver, ldisc::TtyLdisc};

/// all registered devices.
pub(super) static ALL_DEVICES: LazyInit<SpinNoIrq<Vec<Arc<TtyStruct>>>> = LazyInit::new();

/// tty device.
#[derive(Debug)]
pub struct TtyStruct {
    /// driver of device.
    driver: Arc<TtyDriver>,

    /// device's line discipline.
    ldisc: Arc<TtyLdisc>,

    /// index of device.
    index: AtomicUsize,

    /// name of device.
    name: SpinNoIrq<String>,
}

impl TtyStruct {
    pub fn new(driver: Arc<TtyDriver>) -> Self {
        Self {
            driver: driver.clone(),
            ldisc: Arc::new(TtyLdisc::new()),
            index: AtomicUsize::new(0),
            name: SpinNoIrq::new(String::new()),
        }
    }

    /// get tty line discipline.
    pub fn ldisc(&self) -> Arc<TtyLdisc> {
        self.ldisc.clone()
    }

    /// set device index.
    pub fn set_index(&self, index: usize) {
        self.index
            .store(index, core::sync::atomic::Ordering::Relaxed);
    }

    /// set name of device
    pub fn set_name(&self, name: &str) {
        let mut lock = self.name.lock();
        lock.clone_from(&String::from(name));
    }

    /// Convert a tty structure into a name, reflecting the kernel naming policy.
    pub fn name(&self) -> String {
        self.name.lock().clone()
    }

    /// get device's driver.
    pub fn driver(&self) -> Arc<TtyDriver> {
        self.driver.clone()
    }
}

/// called by kernel to get a device.
pub fn get_device_by_name(name: &str) -> Option<Arc<TtyStruct>> {
    let lock = ALL_DEVICES.lock();
    for tty in lock.iter() {
        if tty.name() == name {
            return Some(tty.clone());
        }
    }
    None
}

/// called by kernel to get all devices' names.
/// usually used in init to get the view of tty.
pub fn get_all_device_names() -> Vec<String> {
    let mut ret = vec![];
    let alldev = ALL_DEVICES.lock();
    for dev in alldev.iter() {
        ret.push(dev.name());
    }
    ret
}

/// save a device when registered.
pub fn add_one_device(tty: Arc<TtyStruct>) {
    ALL_DEVICES.lock().push(tty);
}

pub fn init() {
    ALL_DEVICES.init_by(SpinNoIrq::new(vec![]));
}
