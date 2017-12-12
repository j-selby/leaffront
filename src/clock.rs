/// Functions to help with time of day functionality.

use chrono::Local;
use chrono::naive::NaiveTime;
use chrono::Duration as cDuration;

pub fn check_night(start_night : u32, end_night : u32) -> bool {
    let time = Local::now();
    let start_time = NaiveTime::from_hms(start_night, 0, 0);
    let end_time = NaiveTime::from_hms(end_night, 0, 0);
    let cur_date = time.naive_local();
    let cur_time = cur_date.time();

    let start_date = if (cur_time < end_time) && !(start_time < end_time) {
        // Early morning
        let start_date = time.date().naive_local();
        let start_date = start_date - cDuration::days(1);
        start_date.and_time(start_time)
    } else {
        time.date().naive_local().and_time(start_time)
    };

    let end_date = if start_time > end_time && !(cur_time < end_time) {
        // End night is on the next day
        let end_date = time.date().naive_local();
        let end_date = end_date + cDuration::days(1);
        end_date.and_time(end_time)
    } else {
        time.date().naive_local().and_time(end_time)
    };

    cur_date > start_date && cur_date < end_date
}