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
struct TextPrinter(String);
struct ImagePrinter(PhotoSize, Option<String>);
struct StickerPrinter(Sticker);
struct QrCodePrinter(String);
struct BarcodePrinter(BarcodeType, String);

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
    async fn print(&self, print_service: PrintService, bot: Bot) -> HandlerResult {
        log::info!("Preparing Print...");
        let mut cloned_printer = print_service.printer.clone();
        let mut printer = cloned_printer.init()?;

        self.prepare(&mut printer, print_service, bot).await?;

        log::info!("Printing...");
        printer.print()?;
        log::info!("Print complete!");
        Ok(())
    }

    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult;
}

impl Print for TextPrinter {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult {
        printer.writeln(&self.0)?;
        Ok(())
    }
}
impl Print for ImagePrinter {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult {
        let file_id = &self.0.file.id;
        download_and_prepare_printer(
            file_id.to_string(),
            printer,
            bot,
            print_service.image_options,
        )
        .await?;
    
        // Write text if defined
        if let Some(text) = &self.1 {
            printer.writeln(text)?;
        }
        Ok(())
    }
}
impl Print for StickerPrinter {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult {
        let file_id = &self.0.file.id;
        download_and_prepare_printer(
            file_id.to_string(),
            printer,
            bot,
            print_service.image_options,
        )
        .await?;
        Ok(())
    }
}
impl Print for QrCodePrinter {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult {
        printer.qrcode(&self.0)?;
        Ok(())
    }
}
impl Print for BarcodePrinter {
    async fn prepare(
        &self,
        printer: &mut Printer<AsyncSerialPortDriver>,
        print_service: PrintService,
        bot: Bot,
    ) -> HandlerResult {
        let barcode_type = &self.0;
        let content = &self.1;

        match barcode_type {
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
        };

        Ok(())
    }
}
//#endregion

#[derive(Clone)]
struct PrintService {
    printer: Printer<AsyncSerialPortDriver>,
    image_options: ImageOptions,
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
    let cloned_bot = bot.clone();

    let result: HandlerResult = match cmd {
        Command::Help | Command::Start => {
            cloned_bot
                .send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
            Ok(())
        }
        Command::Qr(content) => {
            QrCodePrinter(content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Ean13(content) => {
            BarcodePrinter(BarcodeType::Ean13, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Ean8(content) => {
            BarcodePrinter(BarcodeType::Ean8, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Upca(content) => {
            BarcodePrinter(BarcodeType::Upca, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Upce(content) => {
            BarcodePrinter(BarcodeType::Upce, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Code39(content) => {
            BarcodePrinter(BarcodeType::Code39, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Codabar(content) => {
            BarcodePrinter(BarcodeType::Codabar, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
        Command::Itf(content) => {
            BarcodePrinter(BarcodeType::Itf, content)
                .print(print_service, cloned_bot)
                .await?;
            Ok(())
        }
    };

    if let Err(error) = &result {
        bot.send_message(msg.chat.id, error.to_string()).await?;
    }

    result
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
    let cloned_bot = bot.clone();

    let has_image = msg.photo().is_some_and(|i| i.len() > 0);
    let has_text = msg.text().is_some();
    let has_static_sticker = msg.sticker().is_some_and(|s| s.is_static());

    let result: HandlerResult = match (has_image, has_text, has_static_sticker) {
        (true, _, _) => {
            if let Some(i) = msg.photo().and_then(|i| i.first()) {
                ImagePrinter(i.clone(), msg.text().map(|v| v.to_string()))
                    .print(print_service, cloned_bot)
                    .await?;
            };
            Ok(())
        }
        (false, true, _) => {
            if let Some(v) = msg.text() {
                TextPrinter(v.to_string())
                    .print(print_service, cloned_bot)
                    .await?;
            };
            Ok(())
        }
        (_, _, true) => {
            if let Some(s) = msg.sticker() {
                StickerPrinter(s.clone())
                    .print(print_service, cloned_bot)
                    .await?;
            };
            Ok(())
        }
        (_, _, _) => {
            cloned_bot
                .send_message(msg.chat.id, "Unsupported Type.".to_string())
                .await?;
            Ok(())
        }
    };

    if let Err(ref error) = result {
        bot.send_message(msg.chat.id, error.to_string()).await?;
    }

    result
}
