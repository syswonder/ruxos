/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, Once};

use crate::{self as ruxtask, current, WaitQueue};

static INIT: Once = Once::new();
static SERIAL: Mutex<()> = Mutex::new(());

#[test]
fn test_sched_fifo() {
    let _lock = SERIAL.lock();
    INIT.call_once(ruxtask::init_scheduler);

    const NUM_TASKS: usize = 10;
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

    for i in 0..NUM_TASKS {
        ruxtask::spawn_raw(
            move || {
                println!("sched_fifo: Hello, task {}! ({})", i, current().id_name());
                ruxtask::yield_now();
                let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
                assert_eq!(order, i); // FIFO scheduler
            },
            format!("T{}", i),
            0x1000,
        );
    }

    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        ruxtask::yield_now();
    }
}

#[test]
fn test_fp_state_switch() {
    let _lock = SERIAL.lock();
    INIT.call_once(ruxtask::init_scheduler);

    const NUM_TASKS: usize = 5;
    const FLOATS: [f64; NUM_TASKS] = [
        3.141592653589793,
        2.718281828459045,
        -1.4142135623730951,
        0.0,
        0.618033988749895,
    ];
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

    for (i, float) in FLOATS.iter().enumerate() {
        ruxtask::spawn(move || {
            let mut value = float + i as f64;
            ruxtask::yield_now();
            value -= i as f64;

            println!("fp_state_switch: Float {} = {}", i, value);
            assert!((value - float).abs() < 1e-9);
            FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
        });
    }
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        ruxtask::yield_now();
    }
}

#[test]
fn test_wait_queue() {
    let _lock = SERIAL.lock();
    INIT.call_once(ruxtask::init_scheduler);

    const NUM_TASKS: usize = 10;

    static WQ1: WaitQueue = WaitQueue::new();
    static WQ2: WaitQueue = WaitQueue::new();
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    for _ in 0..NUM_TASKS {
        ruxtask::spawn(move || {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            println!("wait_queue: task {:?} started", current().id());
            WQ1.notify_one(true); // WQ1.wait_until()
            WQ2.wait();

            assert!(!current().in_wait_queue());

            COUNTER.fetch_sub(1, Ordering::Relaxed);
            println!("wait_queue: task {:?} finished", current().id());
            WQ1.notify_one(true); // WQ1.wait_until()
        });
    }

    println!("task {:?} is waiting for tasks to start...", current().id());
    WQ1.wait_until(|| COUNTER.load(Ordering::Relaxed) == NUM_TASKS);
    assert_eq!(COUNTER.load(Ordering::Relaxed), NUM_TASKS);
    assert!(!current().in_wait_queue());
    WQ2.notify_all(true); // WQ2.wait()

    println!(
        "task {:?} is waiting for tasks to finish...",
        current().id()
    );
    WQ1.wait_until(|| COUNTER.load(Ordering::Relaxed) == 0);
    assert_eq!(COUNTER.load(Ordering::Relaxed), 0);
    assert!(!current().in_wait_queue());
}

#[test]
fn test_task_join() {
    let _lock = SERIAL.lock();
    INIT.call_once(ruxtask::init_scheduler);

    const NUM_TASKS: usize = 10;
    let mut tasks = Vec::with_capacity(NUM_TASKS);

    for i in 0..NUM_TASKS {
        tasks.push(ruxtask::spawn_raw(
            move || {
                println!("task_join: task {}! ({})", i, current().id_name());
                ruxtask::yield_now();
                ruxtask::exit(i as _);
            },
            format!("T{}", i),
            0x1000,
        ));
    }

    for i in 0..NUM_TASKS {
        assert_eq!(tasks[i].join(), Some(i as _));
    }
}
