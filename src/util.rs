use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::sync::mpsc::RecvError;

pub(crate) fn big_end_first_4(n: u32, buf: &mut [u8; 4]) {
	buf[0] = (n >> 24) as u8;
	buf[1] = (n >> 16) as u8;
	buf[2] = (n >> 8) as u8;
	buf[3] = (n >> 0) as u8;
}

pub(crate) fn u32x2_of_buf(buf: &[u8; 8]) -> (u32, u32) {
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

pub(crate) fn io_error(recv_error: RecvError) -> io::Error {
	io::Error::new(ErrorKind::Other, recv_error.to_string())
}

pub(crate) fn io_error_of_box(error: Box<dyn Error>) -> io::Error {
	io::Error::new(ErrorKind::Other, error.to_string())
}
