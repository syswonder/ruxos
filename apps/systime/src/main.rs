/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::time::Instant;

struct DateTime {
    year: u64,
    mon: u64,
    mday: u64,
    hour: u64,
    min: u64,
    sec: u64,
}

fn convert_unix_time(unix_time: u64) -> DateTime {
    let secs = unix_time;

    let t = secs;
    let mut tdiv = t / 86400;
    let mut tt = t % 86400;
    let hour = tt / 3600;
    println!("{},{}", tt, hour);
    tt %= 3600;
    let min = tt / 60;
    tt %= 60;
    let sec = tt;
    let mut year = 1970;
    let mut mon = 1;

    while tdiv >= 365 {
        let days = if is_leap_year(year) { 366 } else { 365 };
        if tdiv >= days {
            tdiv -= days;
            year += 1;
        } else {
            break;
        }
    }

    while tdiv > 0 {
        let days = days_in_month(mon, year);
        if tdiv >= days {
            tdiv -= days;
            mon += 1;
        } else {
            break;
        }
    }

    let mday = tdiv + 1;

    DateTime {
        year,
        mon,
        mday,
        hour,
        min,
        sec,
    }
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u64, year: u64) -> u64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("test systime");
    let instant1 = Instant::now();
    let time1 = instant1.current_time();
    println!("time1 {:?}", time1);
    let instant2 = Instant::now();
    let time2 = instant2.current_time();
    println!("time2 {:?}", time2);
    let instant3 = Instant::now();
    let time3 = instant3.current_time().as_secs();
    println!("time3 {:?}", time3);
    let date = convert_unix_time(time3);
    println!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year, date.mon, date.mday, date.hour, date.min, date.sec
    );
}
