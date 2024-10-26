mod io;
mod util;

use dotenv::dotenv;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use image::{
    imageops::{BiLevel, ColorMap},
    DynamicImage, EncodableLayout, GenericImageView, GrayImage, ImageBuffer, Luma, LumaA,
};
use io::driver::AsyncSerialPortDriver;
use std::path::Path;
use std::time::Duration;
use std::{error::Error, io::BufWriter};
use std::{fmt::Debug, u8};
use teloxide::net::Download;
use teloxide::types::{Me, PhotoSize, Sticker};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::fs::File;

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

const AUTHORIZED_USER_ENV_VAR_KEY: &str = "AUTHORIZED_USER";

/// These commands are supported:
#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text
    Help,
    /// Start
    Start,
    /// Print a QR Code
    Qr(String),

    /// Print a EAN13 Barcode (Eg. 978020137962)
    Ean13(String),
    /// Print a EAN8 Barcode (Eg. 9031101)
    Ean8(String),
    /// Print a UPCA Barcode (Eg. 72527273070)
    Upca(String),
    /// Print a UPCE Barcode (Eg. 0123456)
    /// Not supported on EM5820
    Upce(String),
    /// Print a CODE39 Barcode (Eg. ABC-1234)
    /// Only supports short codes on EM5820
    Code39(String),
    /// Print a CODABAR Barcode (Eg. 0123456789)
    /// Not supported on EM5820
    Codabar(String),
    /// Print a ITF Barcode (Eg. 102938475638)
    Itf(String),
}

//#region Print Stuff
struct PrintTypeText(String);
struct PrintTypeImage(PhotoSize, Option<String>);
struct PrintTypeSticker(Sticker);
struct PrintTypeQr(String);
struct PrintTypeBarcode(BarcodeType, Option<BarcodeOption>, String);

enum PrintType {
    /// Just Text
    Text(PrintTypeText),
    /// Image with optional text (to be printed below the image)
    Image(PrintTypeImage),
    /// Sticker
    Sticker(PrintTypeSticker),
    /// QR Code
    Qr(PrintTypeQr),
    /// Barcode
    Barcode(PrintTypeBarcode),
}

enum BarcodeType {
    Ean13,
    Ean8,
    Upca,
    Upce,
    Code39,
    Codabar,
    Itf,
}

trait PreparePrintCommand {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        bot: Bot,
    ) -> HandlerResult;
}

impl PreparePrintCommand for PrintTypeText {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        bot: Bot,
    ) -> HandlerResult {
        printer.writeln(&self.0)?;

        Ok(())
    }
}
impl PreparePrintCommand for PrintTypeImage {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        bot: Bot,
    ) -> HandlerResult {
        todo!();

        if let Some(text) = &self.1 {
            printer.writeln(text)?;
        }

        Ok(())
    }
}
impl PreparePrintCommand for PrintTypeSticker {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        bot: Bot,
    ) -> HandlerResult {
        // Construct path
        let base_path = Path::new("./tmp"); // TODO: Put base_path somewhere else

        let file_id = &self.0.file.id;
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
}
impl PreparePrintCommand for PrintTypeQr {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        _bot: Bot,
    ) -> HandlerResult {
        printer.qrcode(&self.0)?;

        Ok(())
    }
}
impl PreparePrintCommand for PrintTypeBarcode {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        _bot: Bot,
    ) -> HandlerResult {
        let b_type = &self.0;
        let content = &self.2;

        // TODO: Use BarcodeOption (need to deref somehow?)

        match b_type {
            BarcodeType::Ean13 => {
                printer.ean13(&content)?;
            }
            BarcodeType::Ean8 => {
                printer.ean8(&content)?;
            }
            BarcodeType::Upca => {
                printer.upca(&content)?;
            }
            BarcodeType::Upce => {
                printer.upce(&content)?;
            }
            BarcodeType::Code39 => {
                printer.code39(&content)?;
            }
            BarcodeType::Codabar => {
                printer.codabar(&content)?;
            }
            BarcodeType::Itf => {
                printer.itf(&content)?;
            }
        }

        Ok(())
    }
}
//#endregion

//#region Other stuff
//#endregion Other stuff

#[derive(Clone)]
struct PrintService {
    printer: Printer<AsyncSerialPortDriver>,
}

impl PrintService {
    async fn print(&self, bot: Bot, print_type: PrintType) -> HandlerResult {
        let mut cloned_printer = self.printer.clone();
        let printer = cloned_printer.init()?;

        log::info!("Preparing print queue");

        // TODO: Cleaner way to handle this?
        match print_type {
            PrintType::Text(print_type_text) => {
                print_type_text.prepare(printer, bot).await?;
            }
            PrintType::Image(print_type_image) => {
                print_type_image.prepare(printer, bot).await?;
            }
            PrintType::Sticker(print_type_sticker) => {
                print_type_sticker.prepare(printer, bot).await?;
            }
            PrintType::Qr(print_type_qr) => {
                print_type_qr.prepare(printer, bot).await?;
            }
            PrintType::Barcode(print_type_barcode) => {
                print_type_barcode.prepare(printer, bot).await?;
            }
        };

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");

        Ok(())
    }
}

impl From<Printer<AsyncSerialPortDriver>> for PrintService {
    fn from(value: Printer<AsyncSerialPortDriver>) -> Self {
        PrintService { printer: value }
    }
}

#[tokio::main]
async fn main() -> HandlerResult {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    let authorized_user = UserId(
        dotenv::var(AUTHORIZED_USER_ENV_VAR_KEY)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0u64),
    );

    log::info!("Authorized UserID: {}", authorized_user.to_string());

    let driver = AsyncSerialPortDriver::open("/dev/ttyUSB0", 9600, Some(Duration::from_secs(5)))?;
    let printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let print_service = PrintService::from(printer);

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter(|msg: Message, authorized_user: UserId| msg.chat.id != authorized_user)
                .endpoint(handle_unauthorized_user),
        )
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(
            Update::filter_message()
                .filter(|msg: Message| msg.text().is_some_and(|s| s.starts_with('/')))
                .endpoint(handle_unknown_command),
        )
        .branch(Update::filter_message().endpoint(handle_other));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![print_service, authorized_user])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    print_service: PrintService,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let print_type: Option<PrintType> = match cmd {
        Command::Help | Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
            None
        }
        Command::Qr(content) => Some(PrintType::Qr(PrintTypeQr(content))),

        Command::Ean13(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Ean13,
            None,
            content,
        ))),
        Command::Ean8(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Ean8,
            None,
            content,
        ))),
        Command::Upca(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Upca,
            None,
            content,
        ))),
        Command::Upce(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Upce,
            None,
            content,
        ))),
        Command::Code39(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Code39,
            None,
            content,
        ))),
        Command::Codabar(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Codabar,
            None,
            content,
        ))),
        Command::Itf(content) => Some(PrintType::Barcode(PrintTypeBarcode(
            BarcodeType::Itf,
            None,
            content,
        ))),
    };

    if let Some(print_type) = print_type {
        let result = print_service.print(bot.clone(), print_type).await;

        if let Err(error) = result {
            // Send the error message before bubbling the error
            bot.send_message(msg.chat.id, error.to_string()).await?;
            Err(error)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

async fn handle_unknown_command(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.send_message(msg.chat.id, "Unknown Command! Check /help for all commands")
        .await?;
    Ok(())
}

async fn handle_unauthorized_user(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let username = msg.chat.username().unwrap_or("???").to_string();
    let user_id = msg.chat.id.to_string();

    log::info!(
        "[UNAUTHORIZED] User @{} ({}) attempted access to the printer.",
        username,
        user_id,
    );
    bot.send_message(msg.chat.id, "Unauthorized User!").await?;
    Ok(())
}

async fn handle_other(
    bot: Bot,
    msg: Message,
    print_service: PrintService,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let has_image = msg.photo().is_some_and(|i| i.len() > 0);
    let has_text = msg.text().is_some();
    let has_static_sticker = msg.sticker().is_some_and(|s| s.is_static());

    let print_type: Option<PrintType> = match (has_image, has_text, has_static_sticker) {
        (true, _, _) => msg.photo().and_then(|i| i.first()).map(|i| {
            PrintType::Image(PrintTypeImage(i.clone(), msg.text().map(|v| v.to_string())))
        }),
        (false, true, _) => msg
            .text()
            .map(|v| PrintType::Text(PrintTypeText(v.to_string()))),
        (_, _, true) => msg
            .sticker()
            .map(|s| PrintType::Sticker(PrintTypeSticker(s.clone()))),
        (_, _, _) => None,
    };

    match print_type {
        Some(print_type) => {
            let result = print_service.print(bot.clone(), print_type).await;

            if let Err(error) = result {
                // Send the error message before bubbling the error
                bot.send_message(msg.chat.id, error.to_string()).await?;
                Err(error)
            } else {
                Ok(())
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Unsupported Format!").await?;
            Ok(())
        }
    }
}
