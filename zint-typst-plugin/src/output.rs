use serde::Serialize;
use std::{io::Cursor, path::Display};

#[derive(Serialize)]
#[serde(transparent)]
pub struct ErrorDetail {
    pub message: String,
}

impl<E: std::fmt::Display + core::error::Error> From<E> for ErrorDetail {
    fn from(value: E) -> Self {
        ErrorDetail {
            message: value.to_string(),
        }
    }
}

impl std::fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[macro_export]
macro_rules! error {
    ($($t: tt)*) => {
        crate::output::ErrorDetail {
          message: format!($($t)*),
        }
    };
}

pub fn pack_result<T, E>(value: std::result::Result<T, E>) -> Vec<u8>
where
    T: Serialize,
    E: std::fmt::Display,
{
    let mut result = Vec::with_capacity(512);

    let value = match value {
        Ok(it) => Ok(it),
        Err(err) => Err(ErrorDetail {
            message: err.to_string(),
        }),
    };
    ciborium::into_writer(&value, Cursor::new(&mut result)).unwrap();
    result.shrink_to_fit();
    result
}
