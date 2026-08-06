use esp_hal::Async;
use defmt::info;
use embassy_time::{Duration, Timer};

pub struct MmWave<'ch> {
    pin: esp_hal::gpio::Input<'ch>,
    uart: esp_hal::uart::Uart<'ch, Async>,
}

impl<'ch> MmWave<'ch> {
    pub fn new(pin: esp_hal::gpio::Input<'ch>, uart: esp_hal::uart::Uart<'ch, Async>) -> Self {
        Self { pin, uart }
    }

    pub async fn write(&mut self, _test: &[u8]) {
        // Here you would implement the logic to send data to the mmWave sensor
        // For example, you might send a command over UART to set the color
        // let command = format!("SET_COLOR {} {} {}\n", pixels.r, pixels.g, pixels.b);
        // self.uart.write(command.as_bytes()).await.unwrap();
    }

    pub async fn read(&mut self) -> Option<u8> {
        let mut byte = [0u8; 1];
        match self.uart.read_async(&mut byte).await {
            Ok(_) => Some(byte[0]),
            Err(_) => None,
        }
    }

    pub fn get_pin(&self) -> &esp_hal::gpio::Input<'ch> {
        &self.pin
    }
}

#[embassy_executor::task]
pub async fn uart_rx_task(mut mmwave: MmWave<'static>) -> ! {
    // let mut byte = [0u8; 1];
    let mut pin_status = false;
    let test_data: [u8; 3] = [b'x', b'y', b'z']; // Example test data to send

    info!("Starting UART RX task for mmWave sensor");
    loop {
        info!("Waiting for mmWave sensor data...");
        // if let Some(b) = mmwave.read().await {
        //     info!("UART RX byte: {}", b);
        // }
        pin_status = mmwave.get_pin().is_high();
        info!("Pin status: {}", pin_status);
        mmwave.write(&test_data).await;
        Timer::after(Duration::from_secs(10)).await;

    }
}
