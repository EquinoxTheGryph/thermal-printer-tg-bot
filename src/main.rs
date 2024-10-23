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
use teloxide::{
    dispatching::dialogue::GetChatId,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};
use tokio::sync::{Mutex, RwLock};

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

// #[derive(Clone)]
// struct PrintService {
//     printer: Printer<AsyncSerialPortDriver>,
// }

// impl PrintService {
//     fn print_text(&mut self, text: &str) -> HandlerResult {
//         // log::info!("Printing stuff! {}", text);

//         let mut init = self.printer.init()?;

//         init.writeln(text)?.print_cut();

//         Ok(())
//     }
// }

// impl From<Printer<AsyncSerialPortDriver>> for PrintService {
//     fn from(value: Printer<AsyncSerialPortDriver>) -> Self {
//         PrintService { printer: value }
//     }
// }

#[tokio::main]
async fn main() -> HandlerResult {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    let driver = AsyncSerialPortDriver::open("/dev/ttyUSB0", 9600, Some(Duration::from_secs(5)))?;
    let printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    // let print_service = PrintService::from(printer);

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
    if let Some(text) = msg.text() {
        bot.send_message(msg.chat.id, format!("You said a thing! \"{}\"", text))
            .await?;

        print_service.print_text(text);
    }

    Ok(())
}

// async fn simple_commands_handler(
//     bot: Bot,
//     msg: Message,
//     // me: teloxide::types::Me,
//     // cmd: Command,
//     // cfg: PrintService,
// ) -> Result<(), teloxide::RequestError> {
//     log::info!("HANDLE COMMAND");

//     Ok(())
// }

// // async fn simple_messages_handler(
// //     cfg: ConfigParameters,
// //     bot: Bot,
// //     me: teloxide::types::Me,
// //     msg: Message,
// //     cmd: SimpleCommand,
// // ) -> HandlerResult {
// //     log::info!("HANDLE MSGS");

// //     Ok(())
// // }

// /// Parse the text wrote on Telegram and check if that text is a valid command
// /// or not, then match the command. If the command is `/start` it writes a
// /// markup with the `InlineKeyboardMarkup`.
// async fn message_handler(
//     bot: Bot,
//     msg: Message,
//     me: Me,
// ) -> Result<(), Box<dyn Error + Send + Sync>> {
//     if let Some(text) = msg.text() {
//         let result = BotCommands::parse(text, me.username());

//         if let Ok(command) = result {
//             match command {
//                 Command::Help | Command::Start => {
//                     bot.send_message(msg.chat.id, Command::descriptions().to_string())
//                         .await?;
//                 }
//                 Command::BarCode { code_type, content } => {
//                     bot.send_message(msg.chat.id, "TODO!").await?;
//                 }
//                 Command::QrCode(content) => {
//                     bot.send_message(msg.chat.id, "TODO!").await?;
//                 }
//             };
//         } else {
//             if !text.starts_with('/') {
//                 bot.send_message(
//                     msg.chat.id,
//                     format!("You said: {}", msg.text().unwrap_or("NONE")),
//                 )
//                 .await?;
//             } else {
//                 bot.send_message(msg.chat.id, "Invalid Command!").await?;
//             }
//         }
//     }

//     Ok(())
// }
