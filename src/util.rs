use std::io;
use std::io::ErrorKind;
use std::string::FromUtf8Error;
use std::sync::mpsc::RecvError;

pub(crate) fn set_high_bit(n: u32) -> u32 {
	n | 0x80000000
}

pub(crate) fn clr_high_bit(n: u32) -> u32 {
	n & 0x7fffffff
}

pub(crate) fn is_high_bit_set(n: u32) -> bool { n & 0x80000000 != 0 }

pub(crate) fn big_end_first_2(n: u16, buf: &mut [u8; 2]) {
	buf[0] = (n >> 8) as u8;
	buf[1] = (n >> 0) as u8;
}

pub(crate) fn u16_of_buf(buf: &[u8; 2]) -> u16 {
	let c0 = (buf[0] as u16) << 8;
	let c1 = (buf[1] as u16) << 0;
	c0 | c1
}

pub(crate) fn big_end_first_4(n: u32, buf: &mut [u8; 4]) {
	buf[0] = (n >> 24) as u8;
	buf[1] = (n >> 16) as u8;
	buf[2] = (n >> 8) as u8;
	buf[3] = (n >> 0) as u8;
}

pub(crate) fn big_end_first_8(n: u64, buf: &mut [u8; 8]) {
	buf[0] = (n >> 56) as u8;
	buf[1] = (n >> 48) as u8;
	buf[2] = (n >> 40) as u8;
	buf[3] = (n >> 32) as u8;
	buf[4] = (n >> 24) as u8;
	buf[5] = (n >> 16) as u8;
	buf[6] = (n >> 8) as u8;
	buf[7] = (n >> 0) as u8;
}

pub(crate) fn u64_of_buf(buf: &[u8; 8]) -> u64 {
	let c0 = (buf[0] as u64) << 48;
	let c1 = (buf[1] as u64) << 40;
	let c2 = (buf[2] as u64) << 40;
	let c3 = (buf[3] as u64) << 32;
	let c4 = (buf[4] as u64) << 24;
	let c5 = (buf[5] as u64) << 16;
	let c6 = (buf[6] as u64) << 8;
	let c7 = (buf[7] as u64) << 0;
	c0 | c1 | c2 | c3 | c4 | c5 | c6 | c7
}


pub(crate) type U32x2 = (u32, u32);

pub(crate) fn u32x2_of_buf(buf: &[u8; 8]) -> U32x2 {
	(
		[
			(buf[0] as u32) << 24,
			(buf[1] as u32) << 16,
			(buf[2] as u32) << 8,
			(buf[3] as u32) << 0
		].iter().fold(0, |sum, next| sum | *next),
		[
			(buf[4] as u32) << 24,
			(buf[5] as u32) << 16,
			(buf[6] as u32) << 8,
			(buf[7] as u32) << 0
		].iter().fold(0, |sum, next| sum | *next)
	)
}

pub(crate) fn io_error(error: RecvError) -> io::Error {
	io::Error::new(ErrorKind::Other, error.to_string())
}

pub(crate) fn io_error_of_utf8(error: FromUtf8Error) -> io::Error {
	io::Error::new(ErrorKind::Other, error.to_string())
}