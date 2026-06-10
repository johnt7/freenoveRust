#![no_std]

unsafe extern "C" {
	fn add_numbers(a: i32, b: i32) -> i32;
}

pub fn add_numbers_from_c(a: i32, b: i32) -> i32 {
	unsafe { add_numbers(a, b) }
}
