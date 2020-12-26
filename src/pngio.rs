use png;
use std::io::{self, Read, Write};
use image::{Image, PixelFormat};

impl Image {

    /// Reads an image from a PNG file.
    pub fn read_png<R: Read>(mut input: R) -> io::Result<Image> {
        let mut v = Vec::new();
        input.read_to_end(&mut v)?;
        let image = Image::decode_from_png(v.as_slice())?;
        Ok(Image {
            format: PixelFormat::PNG,
            width: image.width,
            height: image.height,
            data: v.into_boxed_slice()
        })
    }

    /// Internal function to decode a PNG.
    pub fn decode_from_png<R: Read>(input: R) -> io::Result<Image> {
        let decoder = png::Decoder::new(input);
        let (info, mut reader) = decoder.read_info()?;
        let pixel_format = match info.color_type {
            png::ColorType::RGBA => PixelFormat::RGBA,
            png::ColorType::RGB => PixelFormat::RGB,
            png::ColorType::GrayscaleAlpha => PixelFormat::GrayAlpha,
            png::ColorType::Grayscale => PixelFormat::Gray,
            _ => {
                // TODO: Support other color types.
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                          format!("unsupported PNG color \
                                                   type: {:?}",
                                                  info.color_type)));
            }
        };
        if info.bit_depth != png::BitDepth::Eight {
            // TODO: Support other bit depths.
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      format!("unsupported PNG bit depth: \
                                               {:?}",
                                              info.bit_depth)));

        }
        let mut image = Image::new(pixel_format, info.width, info.height);
        assert_eq!(image.data().len(), info.buffer_size());
        reader.next_frame(image.data_mut())?;
        Ok(image)
    }

    /// Writes the image to a PNG file.
    pub fn write_png<W: Write>(&self, mut output: W) -> io::Result<()> {
        let color_type = match self.format {
            PixelFormat::RGBA => png::ColorType::RGBA,
            PixelFormat::RGB => png::ColorType::RGB,
            PixelFormat::GrayAlpha => png::ColorType::GrayscaleAlpha,
            PixelFormat::Gray => png::ColorType::Grayscale,
            PixelFormat::Alpha => {
                return self.convert_to(PixelFormat::GrayAlpha)
                    .write_png(output);
            },
            PixelFormat::PNG => {
                return output.write(&self.data).map(|_| ());
            },
        };
        let mut encoder = png::Encoder::new(output, self.width, self.height);
        encoder.set_color(color_type);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.data).map_err(|err| match err {
            png::EncodingError::IoError(err) => err,
            png::EncodingError::Format(msg) => {
                io::Error::new(io::ErrorKind::InvalidData, msg.into_owned())
            }
        })
    }
}
