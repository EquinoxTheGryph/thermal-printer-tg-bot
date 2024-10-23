mod io;

use escpos::driver::*;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use io::driver::{self, AsyncSerialPortDriver};
use std::borrow::BorrowMut;
use std::cell::*;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::*;
use std::time::Duration;
use teloxide::types::{PhotoSize, Sticker};
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};
use tokio::sync::RwLock;

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

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
}

enum PrintType {
    /// Just Text
    Text(String),
    /// Image with optional text (to be printed below the image)
    Image(PhotoSize, Option<String>),
    /// Sticker
    Sticker(Sticker),
}

#[derive(Clone)]
struct PrintService {
    printer: Printer<AsyncSerialPortDriver>,
}

impl PrintService {
    fn print_text(&self, text: &str) -> HandlerResult {
        log::info!("Printing \"{}\"!", text);
        self.printer.clone().init()?.writeln(text)?.print_cut()?;
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

    let driver = AsyncSerialPortDriver::open("/dev/ttyUSB0", 9600, Some(Duration::from_secs(5)))?;
    let printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let print_service = PrintService::from(printer);

    // Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()))
    //     .debug_mode(Some(DebugMode::Dec))
    //     .init()?
    //     .smoothing(true)?
    //     .bold(true)?
    //     .underline(UnderlineMode::Single)?
    //     .writeln("Bold underline")?
    //     .justify(JustifyMode::CENTER)?
    //     .reverse(true)?
    //     .bold(false)?
    //     .writeln("Hello world - Reverse")?
    //     .feed()?
    //     .justify(JustifyMode::RIGHT)?
    //     .reverse(false)?
    //     .underline(UnderlineMode::None)?
    //     .size(2, 3)?
    //     .writeln("Hello world - Normal")?
    //     .print_cut()?;

    let bot = Bot::from_env();

    let handler = dptree::entry()
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
        .dependencies(dptree::deps![print_service])
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
    match cmd {
        Command::Help | Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Qr(_) => {
            bot.send_message(msg.chat.id, "TODO!").await?;
        }
    };

    // log::info!("MESSAGE! {}", msg.text().unwrap_or("-"));

    // print_service.print_text("Hello!");

    Ok(())
}

async fn handle_unknown_command(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.send_message(msg.chat.id, "Unknown Command! Check /help for all commands")
        .await?;
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
        (true, has_text, _) => msg
            .photo()
            .and_then(|i| i.first())
            .map(|i| PrintType::Image(i.clone(), msg.text().map(|v| v.to_string()))),
        (false, true, _) => msg.text().map(|v| PrintType::Text(v.to_string())),
        (_, _, true) => msg.sticker().map(|s| PrintType::Sticker(s.clone())),
        (_, _, _) => None,
    };

    // Print first image if there's one
    if let Some(sticker) = msg.sticker() {
        if sticker.is_animated() || sticker.is_video() {
            bot.send_message(msg.chat.id, "Sticker needs to be static!")
                .await?;
            return Ok(()); // Early Return
        }

        bot.send_message(msg.chat.id, "Stickers are not supported yet.")
            .await?;
    }

    // Print first image if there's one
    if let Some(_img) = msg.photo() {
        bot.send_message(msg.chat.id, "Images are not supported yet.")
            .await?;
    }

    // Print text if the message has it
    if let Some(text) = msg.text() {
        bot.send_message(msg.chat.id, format!("Printing \"{}\"!", text))
            .await?;
        // print_service.print_text(text);
    }

    Ok(())
}
