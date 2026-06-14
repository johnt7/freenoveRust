#![no_std]

use esp_hal::{
	Blocking,
	gpio::interconnect::{PeripheralInput, PeripheralOutput},
	uart::{Config, ConfigError, Instance, RxError, TxError, Uart},
};

unsafe extern "C" {
	fn add_numbers(a: i32, b: i32) -> i32;
}

pub fn add_numbers_from_c(a: i32, b: i32) -> i32 {
	unsafe { add_numbers(a, b) }
}

pub struct SerialPort<'d> {
	uart: Uart<'d, Blocking>,
	line_buf: [u8; 128],
	line_len: usize,
}

impl<'d> SerialPort<'d> {
	pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
		let uart = Uart::new(uart, config)?;
		Ok(Self {
			uart,
			line_buf: [0u8; 128],
			line_len: 0,
		})
	}

	pub fn with_pins(
		self,
		rx: impl PeripheralInput<'d>,
		tx: impl PeripheralOutput<'d>,
	) -> Self {
		Self {
			uart: self.uart.with_rx(rx).with_tx(tx),
			line_buf: self.line_buf,
			line_len: self.line_len,
		}
	}

	pub fn write(&mut self, data: &[u8]) -> Result<usize, TxError> {
		self.uart.write(data)
	}

	pub fn flush(&mut self) -> Result<(), TxError> {
		self.uart.flush()
	}

	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, RxError> {
		self.uart.read(buf)
	}

	pub fn read_ready(&mut self) -> bool {
		self.uart.read_ready()
	}

	pub fn read_line(&mut self, out: &mut [u8]) -> Result<Option<usize>, RxError> {
		while self.uart.read_ready() {
			let mut byte = [0u8; 1];
			let read = self.uart.read(&mut byte)?;

			if read == 0 {
				break;
			}

			if byte[0] == b'\n' {
				let mut len = self.line_len;
				if len > 0 && self.line_buf[len - 1] == b'\r' {
					len -= 1;
				}

				let copy_len = core::cmp::min(len, out.len());
				out[..copy_len].copy_from_slice(&self.line_buf[..copy_len]);
				self.line_len = 0;
				return Ok(Some(copy_len));
			}

			if self.line_len < self.line_buf.len() {
				self.line_buf[self.line_len] = byte[0];
				self.line_len += 1;
			} else {
				let copy_len = core::cmp::min(self.line_buf.len(), out.len());
				out[..copy_len].copy_from_slice(&self.line_buf[..copy_len]);
				self.line_len = 0;
				return Ok(Some(copy_len));
			}
		}

		Ok(None)
	}
}
