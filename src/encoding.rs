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
