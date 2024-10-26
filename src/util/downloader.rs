use crate::io::driver::AsyncSerialPortDriver;
use dotenv::dotenv;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use image::{
    imageops::{BiLevel, ColorMap},
    DynamicImage, EncodableLayout, GenericImageView, GrayImage, ImageBuffer, Luma, LumaA,
};
use std::path::Path;
use std::time::Duration;
use std::{error::Error, io::BufWriter};
use std::{fmt::Debug, u8};
use teloxide::net::Download;
use teloxide::types::{Me, PhotoSize, Sticker};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::fs::File;

// TODO: Make these configurable
const BASE_PATH: &str = "./tmp";
const FILE_EXT: &str = ".webp";

const MAX_WIDTH: u32 = 480; // Must be divisible by 8
const MAX_HEIGHT: u32 = u32::MAX;

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

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
) -> HandlerResult {
    // TODO: Make this configurable
    // Construct path
    let base_path = Path::new(BASE_PATH);

    // let file_id = &self.0.file.id;
    let full_path = base_path.join(format!("{}{}", &file_id.to_string(), FILE_EXT));

    let cloned_str = &file_id.clone();
    let _id = substr(
        cloned_str.as_str(),
        cloned_str.len().checked_sub(8).unwrap_or(0),
        None,
    )
    .unwrap_or("?");

    if let Some(path) = &full_path.to_str() {
        // Get the external file data
        log::info!("[{}] Preparing donwload", &_id);
        let file = bot.get_file(file_id).await?;
        let mut dst = File::create(&full_path).await?;

        // Download the file
        log::info!("[{}] Downloading file to \"{}\"", &_id, &path);
        log::debug!("{}", &file.path);
        bot.download_file(&file.path, &mut dst).await?;

        // Load the downloaded file
        log::info!("[{}] Reading downloaded file", &_id);
        let mut image = image::ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;

        // Process the image data (resize)
        log::info!("[{}] Resizing Image", &_id);
        let filter = image::imageops::FilterType::Lanczos3;
        // TODO: Make the max size configurable instead of hard coded
        image = image.resize(MAX_WIDTH, MAX_HEIGHT, filter);

        // Convert to lumaA8
        log::info!("[{}] Converting Image", &_id);
        let mut image2 = image.to_luma_alpha8();
        let mut image3 = GrayImage::from_fn(image.width(), image.height(), |x, y| {
            let [luma, alpha] = image2.get_pixel(x, y).0;

            // Apply a calculation to make the alpha channel always white
            let max = u8::MAX;
            let output_l = max - { max - luma } * { alpha / max };

            Luma::<u8>([output_l])
        });

        // Process the image data (apply dither)
        log::info!("[{}] Applying Dither", &_id);
        image::imageops::dither(&mut image3, &BiLevel);

        log::info!("[{}] Buffering", &_id);
        let mut buf = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        image3.write_with_encoder(enc)?;

        // Print out the image
        log::info!("[{}] Adding to print queue...", &_id);
        printer.bit_image_from_bytes(&buf)?;
    } else {
        log::error!("[{}] Empty path!", &_id);
    }

    Ok(())
}
