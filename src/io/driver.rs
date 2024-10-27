use escpos::driver::Driver;
use escpos::errors::PrinterError;
use escpos::errors::Result;
use serial2::SerialPort;
use std::borrow::BorrowMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

/// Default timeout in seconds for read/write operations
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

#[derive(Clone)]
pub struct AsyncSerialPortDriver {
    path: String,
    port: Arc<Mutex<Box<SerialPort>>>,
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
        let real_timeout = timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS));
        let mut port =
            SerialPort::open(path, baud_rate).map_err(|e| PrinterError::Io(e.to_string()))?;

        port.set_read_timeout(real_timeout)?;
        port.set_write_timeout(real_timeout)?;

        Ok(Self {
            path: path.to_string(),
            port: Arc::new(Mutex::new(Box::new(port))),
        })
    }
}

impl Driver for AsyncSerialPortDriver {
    fn name(&self) -> String {
        format!("Serial port ({})", self.path)
    }

    fn write(&self, data: &[u8]) -> Result<()> {
        self.port
            .try_lock()
            .map_err(|err| PrinterError::Io(err.to_string()))?
            .borrow_mut()
            .write_all(data)?;
        Ok(())
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let binding = self
            .port
            .try_lock()
            .map_err(|err| PrinterError::Io(err.to_string()))?;
        let port = binding;
        Ok(port.read(buf)?)
    }

    fn flush(&self) -> Result<()> {
        Ok(self
            .port
            .try_lock()
            .map_err(|err| PrinterError::Io(err.to_string()))?
            .borrow_mut()
            .flush()?)
    }
}
