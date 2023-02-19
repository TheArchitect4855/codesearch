/// Decode an integer in the range 0x20..0x7f (exclusive, base 95)
pub fn from_ascii_compat(buf: [u8; 5]) -> u32 {
	(buf[0] - 0x20) as u32
		+ (buf[1] - 0x20) as u32 * 95
		+ (buf[2] - 0x20) as u32 * 9025
		+ (buf[3] - 0x20) as u32 * 857_375
		+ (buf[4] - 0x20) as u32 * 81_450_625
}

pub fn is_printable(s: &[u8]) -> bool {
	s.iter()
		.all(|b| (*b > 0x08 && *b < 0x0e) || (*b >= 0x20 && *b < 0x7f))
}

/// Returns true if the bytes in s *could be* part of a valid
/// Unicode code point. This does not mean s is a valid UTF-8
/// string.
pub fn is_utf8(s: &[u8]) -> bool {
	/*
		UTF-8 bytes are prefixed with any of:
		0XXXXXXX
		10XXXXXX
		110XXXXX
		1110XXXX
		11110XXX
	*/
	s.iter().all(|b| {
		(b & 0x80 == 0)
			|| (b & 0xc0 == 0x80)
			|| (b & 0xe0 == 0xc0)
			|| (b & 0xf0 == 0xe0)
			|| (b & 0xf8 == 0xf0)
	})
}

/// Encode an integer in the range 0x20..0x7f (exclusive, base 95)
pub fn to_ascii_compat(mut n: u32) -> [u8; 5] {
	let mut buf = [0; 5];
	for i in 0..buf.len() {
		buf[i] = (n % 95) as u8 + 0x20;
		n /= 95;
	}

	assert_eq!(n, 0);
	buf
}
