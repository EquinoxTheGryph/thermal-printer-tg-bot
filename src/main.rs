mod io;
mod util;

use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use io::driver::AsyncSerialPortDriver;
use std::error::Error;
use std::fmt::Debug;
use std::time::Duration;
use teloxide::types::{PhotoSize, Sticker};
use teloxide::{prelude::*, utils::command::BotCommands};
use util::downloader::{download_and_prepare_printer, ImageOptions};

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

const AUTHORIZED_USER_ENV_VAR_KEY: &str = "AUTHORIZED_USER";
const CONTRAST_ENV_VAR_KEY: &str = "CONTRAST";
const BRIGHTNESS_ENV_VAR_KEY: &str = "BRIGHTNESS";
const BASE_PATH_ENV_VAR_KEY: &str = "BASE_PATH";
const MAX_WIDTH_ENV_VAR_KEY: &str = "MAX_WIDTH";

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

trait Print {
    async fn print(&self, print_service: PrintService, bot: Bot) -> HandlerResult;
}

impl Print for PrintTypeText {
    async fn print(&self, print_service: PrintService, bot: Bot) -> HandlerResult {
        let mut cloned_printer = print_service.printer.clone();
        let printer = cloned_printer.init()?;

        printer.writeln(&self.0)?;

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }
}
impl Print for PrintTypeImage {
    async fn print(&self, print_service: PrintService, bot: Bot) -> HandlerResult {
        let mut cloned_printer = print_service.printer.clone();
        let printer = cloned_printer.init()?;

        let file_id = &self.0.file.id;
        download_and_prepare_printer(
            file_id.to_string(),
            printer,
            bot,
            print_service.image_options,
        )
        .await?;

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }
}
impl Print for PrintTypeSticker {
    async fn print(&self, print_service: PrintService, bot: Bot) -> HandlerResult {
        let mut cloned_printer = print_service.printer.clone();
        let printer = cloned_printer.init()?;

        let file_id = &self.0.file.id;
        download_and_prepare_printer(
            file_id.to_string(),
            printer,
            bot,
            print_service.image_options,
        )
        .await?;

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }
}
impl Print for PrintTypeQr {
    async fn print(&self, print_service: PrintService, _bot: Bot) -> HandlerResult {
        let mut cloned_printer = print_service.printer.clone();
        let printer = cloned_printer.init()?;

        printer.qrcode(&self.0)?;

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }
}
impl Print for PrintTypeBarcode {
    async fn print(&self, print_service: PrintService, _bot: Bot) -> HandlerResult {
        let mut cloned_printer = print_service.printer.clone();
        let printer = cloned_printer.init()?;

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

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }
}
//#endregion

//#region Other stuff
//#endregion Other stuff

#[derive(Clone)]
struct PrintService {
    printer: Printer<AsyncSerialPortDriver>,
    image_options: ImageOptions,
}

impl PrintService {
    async fn print(&self, bot: Bot, print_type: PrintType) -> HandlerResult {
        // let mut printer = self.printer.clone();

        // let printer = &mut cloned_printer;//.init()?

        log::info!("Preparing print queue");

        // TODO: Cleaner way to handle this?
        match print_type {
            PrintType::Text(print_type_text) => {
                print_type_text.print(self.clone(), bot).await?;
            }
            PrintType::Image(print_type_image) => {
                print_type_image.print(self.clone(), bot).await?;
            }
            PrintType::Sticker(print_type_sticker) => {
                print_type_sticker.print(self.clone(), bot).await?;
            }
            PrintType::Qr(print_type_qr) => {
                print_type_qr.print(self.clone(), bot).await?;
            }
            PrintType::Barcode(print_type_barcode) => {
                print_type_barcode.print(self.clone(), bot).await?;
            }
        };

        Ok(())
    }
}

#[tokio::main]
async fn main() -> HandlerResult {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    // Get env fields
    let authorized_user = UserId(
        dotenv::var(AUTHORIZED_USER_ENV_VAR_KEY)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0u64),
    );
    let contrast = dotenv::var(CONTRAST_ENV_VAR_KEY)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0f32);
    let brightness = dotenv::var(BRIGHTNESS_ENV_VAR_KEY)
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0i32);
    let base_path = dotenv::var(BASE_PATH_ENV_VAR_KEY)
        .ok()
        .unwrap_or("./tmp".to_string());
    let max_width = dotenv::var(MAX_WIDTH_ENV_VAR_KEY)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(64u32);

    log::info!("Authorized UserID: {}", authorized_user.to_string());

    let driver = AsyncSerialPortDriver::open("/dev/ttyUSB0", 9600, Some(Duration::from_secs(5)))?;
    let printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let print_service = PrintService {
        printer,
        image_options: ImageOptions {
            contrast,
            brightness,
            base_path,
            max_width,
        },
    };

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
