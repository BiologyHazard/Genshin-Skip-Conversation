// use windows::Win32::Foundation::{LPARAM, WPARAM};

// use crate::include::minwindef::MAKELONG;

#[macro_export]
macro_rules! MAKEWPARAM {
    ($l:expr, $h:expr) => {
        WPARAM(MAKELONG!($l, $h) as usize)
    };
}

#[macro_export]
macro_rules! MAKELPARAM {
    ($l:expr, $h:expr) => {
        LPARAM(MAKELONG!($l, $h) as isize)
    };
}

// #[allow(non_snake_case)]
// pub fn MAKEWPARAM<T, U>(l: T, h: U) -> WPARAM
// where
//     T: Into<u32>,
//     U: Into<u32>,
// {
//     WPARAM(MAKELONG(l, h) as usize)
// }

// #[allow(non_snake_case)]
// pub fn MAKELPARAM<T, U>(l: T, h: U) -> LPARAM
// where
//     T: Into<u32>,
//     U: Into<u32>,
// {
//     LPARAM(MAKELONG(l, h) as isize)
// }
