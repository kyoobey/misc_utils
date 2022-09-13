


// code shamelessly stolen from
// https://github.com/srijs/rust-crc32fast/blob/master/src/lib.rs
// 
// originally by srijs aka Sam Rijs (dual licensed under MIT and Apache v2.0)



#[allow(dead_code)]
pub fn hash (buf: &[u8]) -> u32 {
	let mut h = Hasher::new();
	h.append(buf);
	h.checksum()
}



#[derive(Clone)]
pub struct Hasher {
	amount: u64,
	state: u32
}



const DEFAULT_INIT_STATE: u32 = 0;

impl Hasher {

	pub fn new () -> Self {
		Self {
			amount: 0,
			state: DEFAULT_INIT_STATE
		}
	}

	pub fn seed (&mut self, seed: u32) {
		self.state = seed;
	}

	pub fn append (&mut self, buf: &[u8]) {
		self.amount += buf.len() as u64;
		self.state = crc32_algorithm::update_fast_16(self.state, buf);
	}

	pub fn checksum (&self) -> u32 {
		self.state
	}

	#[allow(dead_code)]
	pub fn reset (&mut self) {
		self.amount = 0;
		self.state = 0;
	}

	#[allow(dead_code)]
	pub fn combine (&mut self, other: &Self) {
		self.amount += other.amount;
		self.state = crc32_algorithm::combine(self.checksum(), other.checksum(), other.amount);
	}

}



// actual algorithm
mod crc32_algorithm {



	// NOTE: This is static instead of const to ensure that indexing into this table
	//	   doesn't result in large memmoves when in debug mode, which can significantly
	//	   impact performance.
	static CRC32_TABLE: [[u32; 256]; 16] = include!("crc_table.rs");

	pub fn update_fast_16 (prev: u32, mut buf: &[u8]) -> u32 {
		const UNROLL: usize = 4;
		const BYTES_AT_ONCE: usize = 16 * UNROLL;

		let mut crc = !prev;

		while buf.len() >= BYTES_AT_ONCE {
			for _ in 0..UNROLL {
				crc = CRC32_TABLE[0x0][buf[0xf] as usize]
					^ CRC32_TABLE[0x1][buf[0xe] as usize]
					^ CRC32_TABLE[0x2][buf[0xd] as usize]
					^ CRC32_TABLE[0x3][buf[0xc] as usize]
					^ CRC32_TABLE[0x4][buf[0xb] as usize]
					^ CRC32_TABLE[0x5][buf[0xa] as usize]
					^ CRC32_TABLE[0x6][buf[0x9] as usize]
					^ CRC32_TABLE[0x7][buf[0x8] as usize]
					^ CRC32_TABLE[0x8][buf[0x7] as usize]
					^ CRC32_TABLE[0x9][buf[0x6] as usize]
					^ CRC32_TABLE[0xa][buf[0x5] as usize]
					^ CRC32_TABLE[0xb][buf[0x4] as usize]
					^ CRC32_TABLE[0xc][buf[0x3] as usize ^ ((crc >> 0x18) & 0xFF) as usize]
					^ CRC32_TABLE[0xd][buf[0x2] as usize ^ ((crc >> 0x10) & 0xFF) as usize]
					^ CRC32_TABLE[0xe][buf[0x1] as usize ^ ((crc >> 0x08) & 0xFF) as usize]
					^ CRC32_TABLE[0xf][buf[0x0] as usize ^ ((crc >> 0x00) & 0xFF) as usize]
				;
				buf = &buf[16..];
			}
		}

		update_slow(!crc, buf)
	}

	pub fn update_slow (prev: u32, buf: &[u8]) -> u32 { 
		let mut crc = !prev;

		for &byte in buf.iter() {
			crc = CRC32_TABLE[0][((crc as u8) ^ byte) as usize] ^ (crc >> 8);
		}

		!crc
	}



	#[allow(dead_code)]
	const GF2_DIM: usize = 32;

	#[allow(dead_code)]
	fn gf2_matrix_times (mat: &[u32; GF2_DIM], mut vec: u32) -> u32 {
		let mut sum = 0;
		let mut idx = 0;
		while vec > 0 {
			if vec & 1 == 1 {
				sum ^= mat[idx];
			}
			vec >>= 1;
			idx += 1;
		}
		sum
	}

	#[allow(dead_code)]
	fn gf2_matrix_square (square: &mut [u32; GF2_DIM], mat: &[u32; GF2_DIM]) {
		for n in 0..GF2_DIM {
			square[n] = gf2_matrix_times(mat, mat[n]);
		}
	}

	#[allow(dead_code)]
	pub fn combine (mut crc1: u32, crc2: u32, mut len2: u64) -> u32 {
		let mut row: u32;
		let mut even = [0u32; GF2_DIM]; // even power-of-two operators
		let mut odd  = [0u32; GF2_DIM]; // odd  power-of-two operators

		// degenerate case (also disallow negative lengths)
		if len2 <= 0 {
			return crc1;
		}

		// put operator for one zero bit in odd
		odd[0] = 0xedb88320; // CRC-32 polynomial, find out more: https://www.youtube.com/watch?v=IHjNdZQreds
		row = 1;
		for n in 1..GF2_DIM {
			odd[n] = row;
			row <<= 1;
		}

		// put operator for two zero bits in even
		gf2_matrix_square(&mut even, &odd);
		
		// put operator for four zero bits in odd
		gf2_matrix_square(&mut odd, &even);

		// apply len2 zeros to crc1 (first square will put the
		// operator for one zero byte, eight zero bits, in
		// even)
		loop {
			// apply zeros operator for this bit of len2
			gf2_matrix_square(&mut even, &odd);
			if len2 & 1 == 1 {
				crc1 = gf2_matrix_times(&even, crc1);
			}
			len2 >>= 1;

			// if no more btis set, then done
			if len2 == 0 {
				break;
			}

			// another iteration of the loop with odd and even swapped
			gf2_matrix_square(&mut odd, &even);
			if len2 & 1 == 1 {
				crc1 = gf2_matrix_times(&odd, crc1);
			}
			len2 >>= 1;

			// if no more btis set, then done
			if len2 == 0 {
				break;
			}			
		}

		// return combined crc
		crc1 ^ crc2
	}



}



#[cfg(test)]
mod tests {

	use super::crc32_algorithm;

	#[test]
	fn crc32_algorithm_slow() {
		assert_eq!(crc32_algorithm::update_slow(0, b""), 0);

		// test vectors from the iPXE project (input and output are bitwise negated)
		assert_eq!(crc32_algorithm::update_slow(!0x12345678, b""), !0x12345678);
		assert_eq!(crc32_algorithm::update_slow(!0xffffffff, b"hello world"), !0xf2b5ee7a);
		assert_eq!(crc32_algorithm::update_slow(!0xffffffff, b"hello"), !0xc9ef5979);
		assert_eq!(crc32_algorithm::update_slow(!0xc9ef5979, b" world"), !0xf2b5ee7a);

		// Some vectors found on Rosetta code
		assert_eq!(crc32_algorithm::update_slow(0, b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"), 0x190A55AD);
		assert_eq!(crc32_algorithm::update_slow(0, b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"), 0xFF6CAB0B);
		assert_eq!(crc32_algorithm::update_slow(0, b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"), 0x91267E8A);
	}

}


