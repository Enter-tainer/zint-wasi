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

pub use crate::{
    debug, info, warn, error, group, end_group
};
