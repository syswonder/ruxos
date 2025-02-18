/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! the first thing a driver should do is registering itself by `register_driver()`,
//! which will allocate an index for this driver.
//!
//! then, driver should register every device it has by `register_device()`,
//! which will allocate an index for this device.

use crate::tty::TtyStruct;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

/// all tty drivers.
/// only be written when registering a driver.
pub(super) static ALL_DRIVERS: LazyInit<SpinNoIrq<Vec<Arc<TtyDriver>>>> = LazyInit::new();

/// the operations a tty driver must implement.
/// passed by driver when registering itself.
#[derive(Debug)]
pub struct TtyDriverOps {
    /// push a char to device.
    pub putchar: fn(u8),
}

/// tty driver.
#[derive(Debug)]
pub struct TtyDriver {
    /// driver operations.
    pub ops: TtyDriverOps,

    /// driver's devices.
    /// TODO: maybe use rwlock for dynamicly adding devices is better.
    ttys: SpinNoIrq<BTreeMap<usize, Arc<TtyStruct>>>,

    /// index of driver.
    index: usize,

    /// name of driver.
    name: String,
}

impl TtyDriver {
    pub fn new(ops: TtyDriverOps, name: &str) -> Self {
        Self {
            ops,
            ttys: SpinNoIrq::new(BTreeMap::new()),
            index: 0,
            name: String::from(name),
        }
    }

    /// add a device, return its index, -1 means failure.
    fn add_one_device(&self, tty: Arc<TtyStruct>) -> isize {
        let mut index = 0;
        if let Some(k) = self.ttys.lock().last_key_value() {
            index = *k.0;
        }

        // set index of device
        tty.set_index(index);

        // set name of device
        let mut name = self.name.clone();
        name.push(core::char::from_digit(index as _, 16).unwrap());
        tty.set_name(&name);

        // save this device
        self.ttys.lock().insert(index, tty.clone());

        // return device's index
        index as _
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// get all devices' name
    pub fn get_all_device_names(&self) -> Vec<String> {
        let mut ret = vec![];
        for (_, tty) in self.ttys.lock().iter() {
            ret.push(tty.name());
        }
        ret
    }

    /// get device
    pub fn get_device_by_name(&self, name: &str) -> Option<Arc<TtyStruct>> {
        for (_, tty) in self.ttys.lock().iter() {
            if tty.name() == name {
                return Some(tty.clone());
            }
        }
        None
    }

    /// get device
    pub fn get_device_by_index(&self, index: usize) -> Option<Arc<TtyStruct>> {
        self.ttys.lock().get(&index).cloned()
    }
}

pub fn init() {
    ALL_DRIVERS.init_by(SpinNoIrq::new(vec![]));
}

/// get driver by index.
pub fn get_driver_by_index(index: usize) -> Option<Arc<TtyDriver>> {
    let lock = ALL_DRIVERS.lock();
    for driver in lock.iter() {
        if driver.index == index {
            return Some(driver.clone());
        }
    }
    None
}

/// called by driver to register itself.
/// return driver's index.
pub fn register_driver(ops: TtyDriverOps, name: &str) -> usize {
    // create a tty driver structure
    let mut driver = TtyDriver::new(ops, name);

    // lock
    let mut lock = ALL_DRIVERS.lock();

    // grant an index to the driver
    let index = lock.len();
    driver.index = index;

    // push
    lock.push(Arc::new(driver));

    // return index
    index
}

/// called by driver to register device.
/// return device's index, or -1 on failure.
pub fn register_device(driver_index: usize) -> isize {
    let mut index = -1;
    // if driver is found
    if let Some(driver) = get_driver_by_index(driver_index) {
        // create a tty structure
        let tty = Arc::new(TtyStruct::new(driver.clone()));

        // save this structure
        index = driver.add_one_device(tty.clone());
        crate::tty::add_one_device(tty.clone());
    }
    index
}
