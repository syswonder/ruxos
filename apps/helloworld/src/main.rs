#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::time::Duration;

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::time::Instant;

struct DateTime{
    year: u64,
    mon: u64, 
    mday: u64, 
    hour: u64, 
    min: u64, 
    sec: u64,
}

fn convert_unix_time(unix_time: u64) -> DateTime {
    // UNIX 时间戳的起始时间（1970-01-01 00:00:00 UTC）
    //const UNIX_EPOCH_SECS: u64 = 2208988800;

    // 计算 UNIX 时间戳的秒数和纳秒部分
    let secs = unix_time;
    let nsecs = 0;

    // 计算日期和时间
    let mut t = secs;
    let mut tdiv = t / 86400;
    let mut tt = t % 86400;
    let mut hour = tt / 3600;
    println!("{},{}",tt,hour);
    tt %= 3600;
    let mut min = tt / 60;
    tt %= 60;
    let sec = tt;
    let mut year = 1970;
    let mut mon = 1;
    let mut mday = 0;

    // 计算年、月和日
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

    mday = tdiv + 1;

    // 格式化日期和时间为字符串
    let formatted_datetime = DateTime { year, mon, mday, hour, min, sec };

    formatted_datetime
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
    println!("Hello, world!");
    let instant1 = Instant::now();
    let time1 = instant1.current_time();
    println!("time1 {:?}",time1);
    //task::sleep(Duration::from_secs(1));
    let instant2 = Instant::now();
    let time2 = instant2.current_time();
    println!("time2 {:?}",time2);
    let instant3 = Instant::now();
    let time3 = instant3.current_time().as_secs();
    println!("time3 {:?}",time3);
    let date = convert_unix_time(time3);
    println!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year, date.mon, date.mday, date.hour, date.min, date.sec
    );
}
