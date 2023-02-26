use std::{
	fmt::Display,
	ops::{
		BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr,
		ShrAssign,
	},
};

/// A variable-length bitmap.
/// Allows various operations such as bitwise AND, OR, XOR, shifts, etc.
#[derive(Clone, Debug)]
pub struct BitMap(Vec<u8>);

/// An iterator over a bitmap.
pub struct BitMapIterator {
	pos: usize,
	vec: Vec<u8>,
}

impl BitMap {
	/// Create a new bitmap with the specified length, in bits.
	/// # Arguments
	/// `len`: The length of the bitmap in bits.
	/// # Returns
	/// A new bitmap, with all bits initialized to `0`/`false`.
	pub fn new(len: usize) -> Self {
		let bytes = (len as f64 / 8.0).ceil() as usize;
		Self(vec![0; bytes])
	}

	/// Returns this bitmap as a byte slice.
	pub fn as_bytes(&self) -> &[u8] {
		return &self.0;
	}

	/// Gets the value at the specified bit.
	/// Panics if `i` is less than `0` or greater than
	/// the bitmap's length.
	pub fn get(&self, i: usize) -> bool {
		let byte = i / 8;
		let bit = i % 8;
		let mask = (1 << bit) as u8;
		self.0[byte] & mask != 0
	}

	/// Sets the specified bit to the given value.
	/// # Arguments
	/// `i`: The bit index to set.
	/// `v`: The value to set the bit to.
	/// Panics if `i` is less than `0` or greater than
	/// the bitmap's length.
	pub fn set(&mut self, i: usize, v: bool) {
		let byte = i / 8;
		let bit = i % 8;
		let mask = (1 << bit) as u8;
		if v {
			self.0[byte] |= mask;
		} else {
			self.0[byte] &= !mask;
		}
	}
}

impl Display for BitMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut buf = String::with_capacity(self.0.len() * 8);
		for b in &self.0 {
			buf.push_str(&format!("{:08b}", b));
		}

		write!(f, "{buf}")
	}
}

impl From<Vec<u8>> for BitMap {
	fn from(value: Vec<u8>) -> Self {
		Self(value)
	}
}

impl IntoIterator for BitMap {
	type Item = bool;
	type IntoIter = BitMapIterator;

	fn into_iter(self) -> Self::IntoIter {
		BitMapIterator {
			pos: 0,
			vec: self.0,
		}
	}
}

impl Iterator for BitMapIterator {
	type Item = bool;

	fn next(&mut self) -> Option<Self::Item> {
		let byte = self.pos / 8;
		if byte >= self.vec.len() {
			return None;
		}

		let bit = self.pos % 8;

		let byte = self.vec[byte];
		let bit = 1 << bit;

		self.pos += 1;
		Some(byte & bit != 0)
	}
}

impl BitAnd<&Self> for BitMap {
	type Output = Self;

	fn bitand(self, rhs: &Self) -> Self::Output {
		let len = usize::max(self.0.len(), rhs.0.len());
		let mut res = Self::new(len);
		for i in 0..len {
			res.0[i] = self.0.get(i).unwrap_or(&0) & rhs.0.get(i).unwrap_or(&0);
		}

		res
	}
}

impl BitAndAssign<&Self> for BitMap {
	fn bitand_assign(&mut self, rhs: &Self) {
		let len = usize::max(self.0.len(), rhs.0.len());
		for i in 0..len {
			if i < self.0.len() {
				self.0[i] &= rhs.0.get(i).unwrap_or(&0);
			} else {
				self.0.push(0);
			}
		}
	}
}

impl BitOr<&Self> for BitMap {
	type Output = Self;

	fn bitor(self, rhs: &Self) -> Self::Output {
		let len = usize::max(self.0.len(), rhs.0.len());
		let mut res = Self::new(len);
		for i in 0..len {
			res.0[i] =
				self.0.get(i).as_deref().unwrap_or(&0) | rhs.0.get(i).as_deref().unwrap_or(&0);
		}

		res
	}
}

impl BitOrAssign<&Self> for BitMap {
	fn bitor_assign(&mut self, rhs: &Self) {
		let len = usize::max(self.0.len(), rhs.0.len());
		for i in 0..len {
			if i < self.0.len() {
				self.0[i] |= rhs.0.get(i).unwrap_or(&0);
			} else {
				self.0.push(rhs.0[i]);
			}
		}
	}
}

impl BitXor<&Self> for BitMap {
	type Output = Self;

	fn bitxor(self, rhs: &Self) -> Self::Output {
		let len = usize::max(self.0.len(), rhs.0.len());
		let mut res = Self::new(len);
		for i in 0..len {
			res.0[i] =
				self.0.get(i).as_deref().unwrap_or(&0) ^ rhs.0.get(i).as_deref().unwrap_or(&0);
		}

		res
	}
}

impl BitXorAssign<&Self> for BitMap {
	fn bitxor_assign(&mut self, rhs: &Self) {
		let len = usize::max(self.0.len(), rhs.0.len());
		for i in 0..len {
			if i < self.0.len() {
				self.0[i] ^= rhs.0.get(i).unwrap_or(&0);
			} else {
				self.0.push(rhs.0[i]);
			}
		}
	}
}

impl Shl<usize> for BitMap {
	type Output = Self;

	fn shl(self, rhs: usize) -> Self::Output {
		let mut res = self.clone();
		res <<= rhs;
		res
	}
}

impl ShlAssign<usize> for BitMap {
	fn shl_assign(&mut self, rhs: usize) {
		let byte_shifts = rhs / u8::BITS as usize;
		let bit_shifts = rhs % u8::BITS as usize;
		for _ in 0..byte_shifts {
			for i in 1..self.0.len() {
				self.0[i - 1] = self.0[i];
			}

			let end = self.0.len() - 1;
			self.0[end] = 0;
		}

		let hi_bits = u8::BITS as usize - bit_shifts;
		let hi_mask = u8::MAX << hi_bits;
		let mut hi = 0;
		for i in 0..self.0.len() {
			if i > 0 {
				self.0[i - 1] |= hi;
			}

			let byte = self.0[i];
			hi = (byte & hi_mask) >> hi_bits;
			self.0[i] <<= bit_shifts;
		}
	}
}

impl Shr<usize> for BitMap {
	type Output = Self;

	fn shr(self, rhs: usize) -> Self::Output {
		let mut res = self.clone();
		res >>= rhs;
		res
	}
}

impl ShrAssign<usize> for BitMap {
	fn shr_assign(&mut self, rhs: usize) {
		let byte_shifts = rhs / u8::BITS as usize;
		let bit_shifts = rhs % u8::BITS as usize;
		for _ in 0..byte_shifts {
			for i in (0..self.0.len() - 1).rev() {
				self.0[i + 1] = self.0[i];
			}

			self.0[0] = 0;
		}

		let hi_bits = u8::BITS as usize - bit_shifts;
		let hi_mask = u8::MAX >> hi_bits;
		let mut hi = 0;
		for i in 0..self.0.len() {
			let byte = self.0[i];
			self.0[i] >>= bit_shifts;
			self.0[i] |= hi;
			hi = (byte & hi_mask) << hi_bits;
		}
	}
}
