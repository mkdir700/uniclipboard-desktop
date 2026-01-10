use image::DynamicImage;
#[cfg(windows)]
use image::GenericImageView;
#[cfg(windows)]
use std::ops::Range;

pub struct PlatformImage {
    pub dynamic_image: DynamicImage,
    pub width: usize,
    pub height: usize,
}

// 显式实现 Send 和 Sync
unsafe impl Send for PlatformImage {}
unsafe impl Sync for PlatformImage {}

impl PlatformImage {
    pub fn new(img: DynamicImage) -> Self {
        let width = img.width();
        let height = img.height();
        Self {
            dynamic_image: img,
            width: width as usize,
            height: height as usize,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let img = image::load_from_memory(bytes).unwrap();
        Self::new(img)
    }

    /// 根据平台将图片转换为字节数组
    /// windows：转换为bitmap格式
    /// macos：转换为png格式
    /// linux：转换为png格式
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_png()
    }

    #[cfg(windows)]
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_bitmap()
    }

    /// Converts the image to png format
    ///
    /// This method is used on non-windows platforms and may be used
    /// in cross-platform code or tests even on Windows build.
    #[cfg_attr(windows, allow(dead_code))]
    pub fn to_png(&self) -> Vec<u8> {
        let rgba_img = self.dynamic_image.to_rgba8();
        rgba_img.to_vec()
    }

    #[cfg(windows)]
    pub fn to_bitmap(&self) -> Vec<u8> {
        //Flipping image, because scan lines are stored bottom to top
        let img = self.dynamic_image.flipv();

        //Getting the header
        let mut byte_vec = self.get_header(img.width(), img.height());

        for (_, _, pixel) in img.pixels() {
            //Setting the pixels, one by one

            let pixel_bytes = pixel.0;
            //One pixel is 4 bytes, BGR and unused
            byte_vec.push(pixel_bytes[2]);
            byte_vec.push(pixel_bytes[1]);
            byte_vec.push(pixel_bytes[0]);
            byte_vec.push(pixel_bytes[3]); //This is unused based on the specifications
        }

        byte_vec
    }

    /// Generates the header for the bitmap from the width and height of the image.
    /// [Resources][http://www.ece.ualberta.ca/~elliott/ee552/studentAppNotes/2003_w/misc/bmp_file_format/bmp_file_format.htm].
    #[cfg(windows)]
    fn get_header(&self, width: u32, height: u32) -> Vec<u8> {
        //Generating the 54 bytes long vector
        let mut vec = vec![0; 54];

        //BM, as written in specifications
        vec[0] = 66;
        vec[1] = 77;

        //File size
        let file_size = width * height * 4 + 54;
        self.set_bytes(&mut vec, &file_size.to_le_bytes(), 2..6);

        //Not used
        self.set_bytes(&mut vec, &0_u32.to_le_bytes(), 6..10);

        //Offset from the beginning of the file to the beginning of the image data
        let offset = 54_u32;
        self.set_bytes(&mut vec, &offset.to_le_bytes(), 10..14);

        //Size of the second part of the header
        let header_size = 40_u32;
        self.set_bytes(&mut vec, &header_size.to_le_bytes(), 14..18);

        //Horizontal width
        let width_bytes = width.to_le_bytes();
        self.set_bytes(&mut vec, &width_bytes, 18..22);

        //Vertical height
        let height_bytes = height.to_le_bytes();
        self.set_bytes(&mut vec, &height_bytes, 22..26);

        //Number of planes
        let planes = 1_u16;
        self.set_bytes(&mut vec, &planes.to_le_bytes(), 26..28);

        //Bits per pixel
        let bits_per_pixel = 32_u16;
        self.set_bytes(&mut vec, &bits_per_pixel.to_le_bytes(), 28..30);

        //Compression type, 0=no compression
        let compression_type = 0_u32;
        self.set_bytes(&mut vec, &compression_type.to_le_bytes(), 30..34);

        //Compressed size of the image, without the size of the header
        //This size is correct

        // let compressed_size = file_size - 54;

        //But when there is no compression (compression type=0), 0 is an allowed size.
        let compressed_size = 0_u32;

        self.set_bytes(&mut vec, &compressed_size.to_le_bytes(), 34..38);

        //Number of pixels / meter, but 0 is allowed
        let horizontal_resoultion = 0_u32;
        self.set_bytes(&mut vec, &horizontal_resoultion.to_le_bytes(), 38..42);

        let vertical_resolution = 0_u32;
        self.set_bytes(&mut vec, &vertical_resolution.to_le_bytes(), 42..46);

        //I guess the last two are for other formats/compression types
        let actually_used_colors = 0_u32;
        self.set_bytes(&mut vec, &actually_used_colors.to_le_bytes(), 46..50);

        let number_of_important_colors = 0_u32;
        self.set_bytes(&mut vec, &number_of_important_colors.to_le_bytes(), 50..54);

        vec
    }

    /// Replaces the bytes of the `to` slice in the specified range with the bytes of the `from` slice.
    #[cfg(windows)]
    fn set_bytes(&self, to: &mut [u8], from: &[u8], range: Range<usize>) {
        for (from_zero_index, i) in range.enumerate() {
            to[i] = from[from_zero_index];
        }
    }
}
