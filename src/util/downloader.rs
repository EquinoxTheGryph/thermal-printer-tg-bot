use crate::io::driver::AsyncSerialPortDriver;
use async_tempfile::TempFile;
use escpos::printer::Printer;
use image::{imageops::BiLevel, GrayImage, Luma};
use std::error::Error;
use std::u8;
use teloxide::net::Download;
use teloxide::prelude::*;

const MAX_HEIGHT: u32 = u32::MAX;

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone)]
pub struct ImageOptions {
    pub contrast: f32,
    pub brightness: i32,
    pub max_width: u32,
}

pub fn substr(s: &str, begin: usize, end: Option<usize>) -> Option<&str> {
    use std::iter::once;
    let mut itr = s.char_indices().map(|(n, _)| n).chain(once(s.len()));
    let begin_byte = itr.nth(begin)?;
    let end_byte = match end {
        Some(end) if begin >= end => begin_byte,
        Some(end) => itr.nth(end - begin - 1)?,
        None => s.len(),
    };
    Some(&s[begin_byte..end_byte])
}

pub async fn download_and_prepare_printer(
    file_id: String,
    printer: &mut Printer<AsyncSerialPortDriver>,
    bot: Bot,
    options: ImageOptions,
) -> HandlerResult {
    let cloned_str = &file_id.clone();
    let _id = substr(
        cloned_str.as_str(),
        cloned_str.len().checked_sub(8).unwrap_or(0),
        None,
    )
    .unwrap_or("?");

    let result = {
        log::info!("[{}] Creating Temporary File", &_id);
        let mut tmp_file = TempFile::new_with_name(&file_id.clone()).await?;

        // Get the external file data
        log::info!("[{}] Getting Metadata", &_id);
        let file = bot.get_file(file_id).await?;

        // Download the file
        log::info!("[{}] Downloading file", &_id);
        bot.download_file(&file.path, &mut tmp_file).await?;

        // Load the downloaded file
        log::info!("[{}] Reading downloaded file", &_id);
        let mut image = image::ImageReader::open(tmp_file.file_path())?
            .with_guessed_format()?
            .decode()?;

        // Process the image data (resize)
        log::info!("[{}] Resizing Image", &_id);
        let filter = image::imageops::FilterType::Lanczos3;
        // TODO: Make the max size configurable instead of hard coded
        image = image.resize(options.max_width, MAX_HEIGHT, filter);

        // Convert to lumaA8
        log::info!("[{}] Converting Image", &_id);
        let image2 = image.to_luma_alpha8();
        let mut image3 = GrayImage::from_fn(image.width(), image.height(), |x, y| {
            let [luma, alpha] = image2.get_pixel(x, y).0;

            // Apply a calculation to make the alpha channel always white
            let max = u8::MAX;
            let output_l = max - { max - luma } * { alpha / max };

            Luma::<u8>([output_l])
        });

        // Process the image data (apply dither)
        log::info!("[{}] Applying Contrast, Brighten and Dither", &_id);
        image::imageops::colorops::contrast_in_place(&mut image3, options.contrast);
        image::imageops::colorops::brighten_in_place(&mut image3, options.brightness);
        image::imageops::dither(&mut image3, &BiLevel);

        log::info!("[{}] Buffering", &_id);
        let mut buf = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        image3.write_with_encoder(enc)?;

        // Print out the image
        log::info!("[{}] Adding to print queue...", &_id);
        printer.bit_image_from_bytes(&buf)?;

        Ok(())
    };

    if let Err(error) = &result {
        log::error!("[{}] Error: {}", &_id, error );
    };

    result
}
