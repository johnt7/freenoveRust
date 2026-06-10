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
}

impl<'d> SerialPort<'d> {
	pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
		let uart = Uart::new(uart, config)?;
		Ok(Self { uart })
	}

	pub fn with_pins(
		self,
		rx: impl PeripheralInput<'d>,
		tx: impl PeripheralOutput<'d>,
	) -> Self {
		Self {
			uart: self.uart.with_rx(rx).with_tx(tx),
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
}
