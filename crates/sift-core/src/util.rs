//! Small helpers: local timestamps without a date-time crate (D22).

use std::time::{SystemTime, UNIX_EPOCH};

/// (year, month, day, hour, minute, second) in local time.
fn local_parts() -> (i64, u32, u32, u32, u32, u32) {
    let secs =
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + local_offset_secs();
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // civil from days (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

#[cfg(windows)]
fn local_offset_secs() -> i64 {
    #[repr(C)]
    struct TimeZoneInformation {
        bias: i32,
        _std_name: [u16; 32],
        _std_date: [u16; 8],
        std_bias: i32,
        _dl_name: [u16; 32],
        _dl_date: [u16; 8],
        dl_bias: i32,
    }
    unsafe extern "system" {
        fn GetTimeZoneInformation(tz: *mut TimeZoneInformation) -> u32;
    }
    let mut tz: TimeZoneInformation = unsafe { std::mem::zeroed() };
    let r = unsafe { GetTimeZoneInformation(&mut tz) };
    let bias = tz.bias + if r == 2 { tz.dl_bias } else { tz.std_bias };
    -(bias as i64) * 60
}

#[cfg(not(windows))]
fn local_offset_secs() -> i64 {
    0
}

/// "20261106-213015" (V0's log and trace names).
pub fn stamp() -> String {
    let (y, m, d, h, mi, s) = local_parts();
    format!("{y:04}{m:02}{d:02}-{h:02}{mi:02}{s:02}")
}

/// "2026-11-06 21:30" (report header).
pub fn now_minutes() -> String {
    let (y, m, d, h, mi, _) = local_parts();
    format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}")
}
