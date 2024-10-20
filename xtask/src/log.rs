#[macro_export]
macro_rules! debug {
    ($msg: expr) => {{
        #[cfg(all(not(ci), debug_assertions))]
        println!("[DEBUG]: {}", $msg);
        #[cfg(ci = "github")]
        println!("::debug :: {}", $msg);
    }};
    ($msg: literal, $($args: expr),*) => {{
        #[cfg(all(not(ci), debug_assertions))]
        println!(concat!["[DEBUG]: ", $msg], $($args),*);
        #[cfg(ci = "github")]
        println!(concat!["::debug :: ", $msg], $($args),*);
    }};
}

#[macro_export]
macro_rules! info {
    ($msg: expr) => {{
        println!("{}", $msg)
    }};
    ($msg: literal, $($args: expr),*) => {{
        println!($msg, $($args),*)
    }};
}

#[macro_export]
macro_rules! warn {
    ($msg: expr) => {{
        #[cfg(not(ci))]
        println!("[WARNING]: {}", $msg);
        #[cfg(ci = "github")]
        println!("::warning :: {}", $msg);
    }};
    ($msg: literal, $($args: expr),*) => {{
        #[cfg(not(ci))]
        println!(concat!["[WARNING]: ", $msg], $($args),*);
        #[cfg(ci = "github")]
        println!(concat!["::warning :: ", $msg], $($args),*);
    }};
}

#[macro_export]
macro_rules! error {
    ($msg: expr) => {{
        #[cfg(not(ci))]
        eprintln!("[ERROR]: {}", $msg);
        #[cfg(ci = "github")]
        eprintln!("::error :: {}", $msg);
    }};
    ($msg: literal, $($args: expr),*) => {{
        #[cfg(not(ci))]
        eprintln!(concat!["[ERROR]: ", $msg], $($args),*);
        #[cfg(ci = "github")]
        eprintln!(concat!["::error :: ", $msg], $($args),*);
    }};
}

#[macro_export]
macro_rules! group {
    ($name: expr) => {{
        #[cfg(ci = "github")]
        println!("::group::{}", $name);
    }};
    ($name: literal, $($args: expr),*) => {{
        #[cfg(ci = "github")]
        println!(concat!["::group::", $name], $($args),*);
    }};
}

#[macro_export]
macro_rules! end_group {
    () => {{
        #[cfg(ci = "github")]
        println!("::endgroup::");
    }};
}

pub struct DisplayDuration {
    pub duration: std::time::Duration,
    pub show_ms: bool,
}

impl Display for DisplayDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DAY_IN_MILLI: u128 = 86_400_000;
        const HOUR_IN_MILLI: u128 = 3_600_000;
        const MINUTE_IN_MILLI: u128 = 60_000;
        const SECOND_IN_MILLI: u128 = 1_000;

        let mut ms = self.duration.as_millis();
        let mut print_any = false;
        let mut take_time = |unit: &'static str, scale: u128| {
            let value = ms / scale;
            ms -= value * scale;
            if value > 0 {
                if print_any {
                    f.write_char(' ')?;
                }
                write!(f, "{}{}", value, unit)?;
                print_any = true;
            };
            Ok(())
        };
        take_time("d", DAY_IN_MILLI)?;
        take_time("h", HOUR_IN_MILLI)?;
        take_time("min", MINUTE_IN_MILLI)?;
        take_time("s", SECOND_IN_MILLI)?;
        if self.show_ms {
            if print_any {
                f.write_char(' ')?;
            }
            write!(f, "{}ms", ms)?;
        }

        Ok(())
    }
}

#[cfg(not(ci = "github"))]
pub const SUMMARY_FILE: Option<&str> = None;
#[cfg(ci = "github")]
pub const SUMMARY_FILE: Option<&str> = Some(env!("GITHUB_STEP_SUMMARY"));

#[macro_export]
macro_rules! summary {
    ($msg: expr) => {{
        $crate::log::summary!("{}", $msg);
    }};
    ($msg: literal, $($args: expr),*) => {{
        if let Some(_summary) = $crate::log::SUMMARY_FILE {
            use std::io::Write;

            let _summary = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(_summary);
            if let Ok(_summary) = _summary {
                let mut _summary = std::io::BufWriter::new(_summary);
                let _ = writeln!(_summary, $msg, $($args),*);
            }
        }
    }};
}

use std::fmt::{Display, Write};

#[allow(unused_imports)]
pub use crate::{debug, end_group, error, group, info, summary, warn};
