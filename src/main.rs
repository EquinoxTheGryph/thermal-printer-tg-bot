use std::time::Duration;

use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use escpos::{driver::*, errors::Result};

fn main() -> Result<()> {
    // env_logger::init();

    // let driver = NetworkDriver::open("192.168.1.248", 9100, None)?;
    // let driver = ConsoleDriver::open(true);
    let driver = SerialPortDriver::open("/dev/ttyUSB0", 9600, Some(Duration::from_secs(5)))?;

    Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()))
        .debug_mode(Some(DebugMode::Dec))
        .init()?
        .smoothing(true)?
        .bold(true)?
        .underline(UnderlineMode::Single)?
        .writeln("Bold underline")?
        .justify(JustifyMode::CENTER)?
        .reverse(true)?
        .bold(false)?
        .writeln("Hello world - Reverse")?
        .feed()?
        .justify(JustifyMode::RIGHT)?
        .reverse(false)?
        .underline(UnderlineMode::None)?
        .size(2, 3)?
        .writeln("Hello world - Normal")?
        .print_cut()?;

    Ok(())
}
