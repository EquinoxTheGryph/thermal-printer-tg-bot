use escpos::driver::Driver;
use escpos::errors::PrinterError;
use escpos::errors::Result;
use serialport::SerialPort;
use std::cell::*;
use std::rc::Rc;
use std::time::Duration;

/// Default timeout in seconds for read/write operations
const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Clone)]
pub struct AsyncSerialPortDriver {
    path: String,
    port: Rc<RefCell<Box<dyn SerialPort>>>,
}

impl AsyncSerialPortDriver {
    /// Open a new Serial port connection
    ///
    /// # Example
    ///
    /// ```no_run
    /// use escpos::printer::Printer;
    /// use escpos::utils::*;
    /// use escpos::driver::*;
    /// use std::time::Duration;
    ///
    /// let driver = AsyncSerialPortDriver::open("/dev/ttyUSB0", 115_200, Some(Duration::from_secs(5))).unwrap();
    /// let mut printer = Printer::new(driver, Protocol::default(), None);
    /// ```
    pub fn open(path: &str, baud_rate: u32, timeout: Option<Duration>) -> Result<Self> {
        let mut port = serialport::new(path, baud_rate);
        if let Some(timeout) = timeout {
            port = port.timeout(timeout);
        }
        let port = port.open().map_err(|e| PrinterError::Io(e.to_string()))?;

        Ok(Self {
            path: path.to_string(),
            port: Rc::new(RefCell::new(port)),
        })
    }
}

impl Driver for AsyncSerialPortDriver {
    fn name(&self) -> String {
        format!("Serial port ({})", self.path)
    }

    fn write(&self, data: &[u8]) -> Result<()> {
        self.port.try_borrow_mut()?.write_all(data)?;

        Ok(())
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut port = self.port.try_borrow_mut()?;
        port.set_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .map_err(|e| PrinterError::Io(e.to_string()))?;
        Ok(port.read(buf)?)
    }

    fn flush(&self) -> Result<()> {
        Ok(self.port.try_borrow_mut()?.flush()?)
    }
}
