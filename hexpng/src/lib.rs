


//! hexpng provides a function to generate solid/translucent png data from a hexcode
//! 
//! thanks [darka](https://darka.github.io/posts/generating-png-in-python)



/// a const representing the version of this library
pub const VERSION: [u8; 3] = [0, 0, 0];



mod crc32;

pub use png::generate_png;



mod png {



	// for compression
	use std::io;
	// use libflate::zlib::{ Encoder, EncodeOptions };
	use libflate::zlib::Encoder;


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
		// let encoder_options = EncodeOptions::new().no_compression();
		// let mut encoder = Encoder::with_options(Vec::new(), encoder_options).unwrap();
		let mut encoder = Encoder::new(Vec::new()).unwrap();
		io::copy(&mut &data[..], &mut encoder).unwrap();
		encoder.finish().into_result().unwrap()
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

	#[test]
	fn test_generate_png_data () {

		// to future me: fix this test please

		let expected_width = 10;
		let expected_height = 10;
		let png_data = super::generate_png(235, 35, 35, 127);

		// use std::io::Write;
		// let mut file = std::fs::File::create("output.png").unwrap();
		// file.write_all(&png_data).unwrap();

		let test_bytes = {
			let s = "89504e470d0a1a0a0000000d494844520000000a0000000a08060000008d32cfbd0000002249444154789cedca211100000c02c07521e8025382061c028978f747e0138b563f0a6a8ba8c177370a0e0000000049454e44ae426082";
			(0..s.len())
				.step_by(2)
				.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
				.collect::<Vec<u8>>()
		};

		// if let image::DynamicImage::ImageRgba8(img) = image::load_from_memory(&test_bytes).unwrap() {
		// 	assert!(img.width() == 10, "png width is {} which doesn't match expected {}", img.width(), expected_width);
		// 	assert!(img.height() == 10, "png height is {} which doesn't match expected {}", img.height(), expected_height);

		// 	for (_, _, pixel) in img.enumerate_pixels() {
		// 		assert!(&image::Rgba::<u8>([235, 35, 35, 127]) == pixel, "pixel doesn't match expected Rgba(235, 35, 35, 127)");
		// 	}
		// } else {
		// 	panic!("png data format is not RGBA8");
		// }

		assert!(&png_data == test_data, "png data doesn't match expected");

	}

}


