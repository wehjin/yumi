pub(crate) fn big_end_first_4(n: u32, buf: &mut [u8; 4]) {
	buf[0] = (n >> 24) as u8;
	buf[1] = (n >> 16) as u8;
	buf[2] = (n >> 8) as u8;
	buf[3] = (n >> 0) as u8;
}

pub(crate) fn u32_of_buf(buf: &[u8; 4]) -> u32 {
	let values = [
		(buf[0] as u32) << 24,
		(buf[1] as u32) << 16,
		(buf[2] as u32) << 8,
		(buf[3] as u32) << 0
	];
	values.iter().fold(0, |sum, next| sum | *next)
}
