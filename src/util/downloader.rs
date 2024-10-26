type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

pub async fn download_and_prepare_printer(
    file_id: String,
    printer: &mut Printer<AsyncSerialPortDriver>,
    bot: Bot,
) -> HandlerResult {
    // Construct path
    let base_path = Path::new("./tmp"); // TODO: Put base_path somewhere else

    // let file_id = &self.0.file.id;
    let full_path = base_path.join(format!("tmp_{}.webp", file_id));

    let _full_id = file_id.to_string();
    let _id = {
        // Get just the last 8 chars
        let count = 8;
        let split_pos = _full_id.char_indices().nth_back(count - 1).unwrap().0;
        &_full_id[split_pos..]
    };

    if let Some(path) = &full_path.to_str() {
        // Get the external file data
        log::info!("[{}] Preparing donwload", &_id);
        let file = bot.get_file(file_id).await?;
        let mut dst = File::create(&full_path).await?;

        // Download the file
        log::info!("[{}] Downloading file to \"{}\"", &_id, &path);
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
        image = image.resize(160, u32::MAX, filter);

        // Convert to lumaA8
        let mut image2 = image.to_luma_alpha8();
        let mut image3 = GrayImage::from_fn(image.width(), image.height(), |x, y| {
            let [l, a] = image2.get_pixel(x, y).0;

            // Black = 0, white = 1
            // Trans = 0, solid = 1

            // [0,0] = [1]
            // [0,1] = [0]
            // [1,1] = [1]
            // [1,0] = [1]

            let mix = |l: u8, a: u8| u8::MAX - { u8::MAX - l } * { a / u8::MAX };

            Luma::<u8>([mix(l, a)])
        });

        // Process the image data (apply dither)
        log::info!("[{}] Applying Dither", &_id);
        image::imageops::dither(&mut image3, &BiLevel);

        log::info!("[{}] Buffering", &_id);
        let mut buf = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        image3.write_with_encoder(enc)?;

        // Print out the image
        // TODO: Make the max size configurable instead of hard coded
        log::info!("[{}] Adding to print queue...", &_id);
        // printer.bit_image_from_bytes_option(
        //     &buf,
        //     BitImageOption::new(Some(160), None, BitImageSize::Normal)?,
        // )?;
        printer.bit_image_from_bytes(&buf)?;

        // // log::info!("PRINT {path:?}")
        // printer.bit_image_option(
        //     path,
        //     BitImageOption::new(Some(400), None, BitImageSize::Normal)?,
        // )?;
        // printer.bit_image_from_bytes(bytes)
    } else {
        log::error!("[{}] Empty path!", &_id);
    }

    Ok(())
}
