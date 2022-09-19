


//! hexpng provides a function to generate solid/translucent png data from a hexcode
//! 
//! thanks [darka](https://darka.github.io/posts/generating-png-in-python)



#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;



/// a const representing the version of this library
pub const VERSION: [u8; 3] = [0, 0, 0];



mod crc32;

pub use png::generate_png;



mod png {

	use compression::prelude::{ Action, ZlibEncoder, EncodeExt };

	#[cfg(not(feature = "std"))]
	use alloc::{ vec, vec::Vec };



	const IMAGE_WIDTH : u32 = 10;
	const IMAGE_HEIGHT: u32 = 10;

	const HEADER: &[u8] = b"\x89PNG\r\n\x1A\n";

	/// function to generate solid/translucent png data from a hexcode
	/// 
	/// as the signature suggests, this function requires 4 u8
	/// values i.e. 0-255 representing the lowest and highest
	/// intensities of the red, green, blue channels and opacity
	/// in the alpha channel respectively
	pub fn generate_png (r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
		HEADER.iter().copied()
			.chain(chunk(b"IHDR", &ihdr_data(IMAGE_WIDTH, IMAGE_HEIGHT, 8, 6)).iter().copied())
			.chain(chunk(b"IDAT", &idat_data(generate_data(r, g, b, a, IMAGE_WIDTH, IMAGE_HEIGHT))).iter().copied())
			.chain(chunk(b"IEND", b"").iter().copied())
			.collect::<Vec<u8>>()
	}


	fn generate_data (r: u8, g: u8, b: u8, a: u8, w: u32, h: u32) -> Vec<u8> {
		vec![[vec![0u8], vec![[r, g, b, a]; w as usize].concat()].concat(); h as usize].concat()
	}

	fn idat_data (data: Vec<u8>) -> Vec<u8> {
		data.encode(&mut ZlibEncoder::new(), Action::Finish).collect::<Result<Vec<u8>, _>>().unwrap()
	}

	// Image Header Chunk
	fn ihdr_data (width: u32, height: u32, bit_depth: u8, color_type: u8) -> Vec<u8> {
		[
			width.to_be_bytes().to_vec(),
			height.to_be_bytes().to_vec(),
			vec![bit_depth],
			vec![color_type], // 6 for RGBA
			vec![0u8; 3] // compression, filter, interlace
		].concat()
	}

	fn chunk (chunk_type: &[u8], data: &[u8]) -> Vec<u8> {
		[
			(data.len() as u32).to_be_bytes().to_vec(),
			chunk_type.to_vec(),
			data.to_vec(),
			chunk_checksum(chunk_type, data).to_be_bytes().to_vec()
		].concat()
	}

	fn chunk_checksum (chunk_type: &[u8], data: &[u8]) -> u32 {
		let mut hasher = super::crc32::Hasher::new();
		let mut checksum: u32;

		hasher.append(chunk_type);
		checksum = hasher.checksum();

		hasher.seed(checksum);
		hasher.append(data);
		checksum = hasher.checksum();

		checksum
	}

}



#[cfg(test)]
mod tests {

	#[cfg(not(feature = "std"))]
	use alloc::{ vec, vec::Vec };

	use compression::prelude::{ ZlibDecoder, DecodeExt };



	#[test]
	fn test_generate_png_data () {

		let expected_width = 10;
		let expected_height = 10;
		let png_data = super::generate_png(235, 35, 35, 127);

		// use std::io::Write;
		// let mut file = std::fs::File::create("output.png").unwrap();
		// file.write_all(&png_data).unwrap();

		let compressed_width  = u32::from_be_bytes(png_data[16..20].try_into().unwrap());
		let compressed_height = u32::from_be_bytes(png_data[20..24].try_into().unwrap());

		assert!(compressed_width == expected_width && compressed_height == expected_height, "png dimensions doesn't match");

		let decompressed_data = png_data[41..png_data.len()-12].to_vec().decode(&mut ZlibDecoder::new()).collect::<Result<Vec<u8>, _>>().unwrap();
		assert!(decompressed_data[..9].to_vec() == vec![0, 235, 35, 35, 127, 235, 35, 35, 127], "png data is corrupted");

	}

}


