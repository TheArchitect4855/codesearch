use std::ffi::{OsStr, OsString};

const HEX_CHARS: [char; 16] = [
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Returns `true` if and only if the bytes in `s` are between the ranges
/// `0x09` (ASCII HT, Horizontal Tab) to `0x0d` (ASCII CR, Carriage Return)
/// and `0x20` (ASCII Space) to `0x7e` (ASCII ~, Tilde).
/// These are both printable ranges.
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

/// Converts `s` into a hexadecimal string.
pub fn to_hex(s: &[u8]) -> String {
	let mut buf = String::with_capacity(s.len() * 2);
	for b in s {
		let hi = (*b & 0xf0) >> 4;
		let lo = *b & 0x0f;
		buf.push(HEX_CHARS[hi as usize]);
		buf.push(HEX_CHARS[lo as usize]);
	}

	buf
}

/// Converts an OS string to a byte array.
#[cfg(target_family = "unix")]
pub fn os_str_to_bytes(s: &OsStr) -> &[u8] {
	use std::os::unix::ffi::OsStrExt;
	s.as_bytes()
}

/// Converts an OS string to a byte array.
#[cfg(target_family = "windows")]
pub fn os_str_to_bytes(s: &OsStr) -> Vec<u8> {
	use std::os::windows::ffi::OsStrExt;
	let mut res = Vec::with_capacity(s.len());
	s.encode_wide().for_each(|v| {
		let bytes = v.to_be_bytes();
		res.extend_from_slice(&bytes);
	});

	res
}

/// Converts a vec of bytes to an OsString.
#[cfg(target_family = "unix")]
pub fn bytes_to_os_string(b: Vec<u8>) -> OsString {
	use std::os::unix::ffi::OsStringExt;
	OsString::from_vec(b)
}

/// Converts a vec of bytes to an OsString.
#[cfg(target_family = "windows")]
pub fn bytes_to_os_string(b: Vec<u8>) -> OsString {
	use std::os::windows::ffi::OsStringExt;
	if b.len() % 2 != 0 {
		panic!("invalid number of bytes for a UTF-16 string");
	}

	let wide = Vec::with_capacity(b.len() / 2);
	let mut buf = [0; 2];
	for i in (0..b.len()).step(2) {
		buf.copy_from_slice(&b[i..i + 2]);
		wide.push(u16::from_be_bytes(buf));
	}

	OsString::from_wide(wide)
}
