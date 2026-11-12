use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LAST_MESSAGE: Mutex<String> = Mutex::new(String::new());

pub fn install() {
    std::panic::set_hook(Box::new(|info| {
        let message = message_of(info.payload());
        let place = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "неизвестно".to_string());
        write(&message, &place, true);
    }));
}

pub fn record(message: &str, place: &str) {
    write(message, place, false);
}

pub fn guard<T>(action: impl FnOnce() -> T) -> Result<T, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)) {
        Ok(value) => Ok(value),
        Err(payload) => {
            let message = message_of(payload.as_ref());
            write(&message, "перехвачено, работа продолжена", false);
            Err(message)
        }
    }
}

fn message_of(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "неизвестная ошибка".to_string()
    }
}

fn write(message: &str, place: &str, always: bool) {
    let mut last = LAST_MESSAGE.lock().unwrap_or_else(|error| error.into_inner());
    if !always && *last == message {
        return;
    }
    *last = message.to_string();
    let path = log_path();
    if let Ok(mut file) = std::fs::File::create(&path) {
        let _ = writeln!(file, "{}\n{place}\n{message}", stamp());
    }
}

fn log_path() -> PathBuf {
    let (day, month, year, hour, minute, second) = local_parts();
    let stem = file_stem(day, month, year, hour, minute, second);
    let dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let direct = dir.join(format!("{stem}.log"));
    if !direct.exists() {
        return direct;
    }
    for index in 2..1000 {
        let candidate = dir.join(format!("{stem}-{index}.log"));
        if !candidate.exists() {
            return candidate;
        }
    }
    direct
}

fn file_stem(day: u32, month: u32, year: u32, hour: u32, minute: u32, second: u32) -> String {
    format!("sjxc-crash-log-{day:02}-{month:02}-{year:02}-{hour:02}-{minute:02}-{second:02}")
}

fn stamp() -> String {
    let (day, month, year, hour, minute, second) = local_parts();
    let full_year = if year < 70 { 2000 + year } else { 1900 + year };
    format!("{full_year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn local_parts() -> (u32, u32, u32, u32, u32, u32) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct SystemTime {
            year: u16,
            month: u16,
            day_of_week: u16,
            day: u16,
            hour: u16,
            minute: u16,
            second: u16,
            milliseconds: u16,
        }
        extern "system" {
            fn GetLocalTime(time: *mut SystemTime);
        }
        let mut time = SystemTime {
            year: 0,
            month: 0,
            day_of_week: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            milliseconds: 0,
        };
        unsafe { GetLocalTime(&mut time) };
        (
            time.day as u32,
            time.month as u32,
            (time.year % 100) as u32,
            time.hour as u32,
            time.minute as u32,
            time.second as u32,
        )
    }
    #[cfg(not(windows))]
    {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let days = (seconds / 86_400) as i64;
        let time = seconds % 86_400;
        let (year, month, day) = civil_from_days(days);
        let hour = time / 3600;
        let minute = (time % 3600) / 60;
        let second = time % 60;
        (day, month, (year % 100) as u32, hour as u32, minute as u32, second as u32)
    }
}

#[cfg(not(windows))]
fn civil_from_days(mut z: i64) -> (i64, u32, u32) {
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let mut year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    if month <= 2 {
        year += 1;
    }
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::file_stem;

    #[test]
    fn crash_file_stem_matches_pattern() {
        assert_eq!(file_stem(12, 11, 26, 17, 58, 3), "sjxc-crash-log-12-11-26-17-58-03");
    }
}
