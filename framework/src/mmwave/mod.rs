pub struct MmWave<'ch> {
    pin: esp_hal::gpio::Input<'ch>,
    uart: esp_hal::uart::Uart<'ch, Dm>,
    status: bool,
}

impl<'ch> MmWave<'ch> {
    pub fn new(pin: esp_hal::gpio::Input<'ch>, uart: esp_hal::uart::Uart<'ch>) -> Self {
        Self { pin, uart, status: false  }
    }

    pub async fn write(&mut self, test: &[u8]) {
        // Here you would implement the logic to send data to the mmWave sensor
        // For example, you might send a command over UART to set the color
        // let command = format!("SET_COLOR {} {} {}\n", pixels.r, pixels.g, pixels.b);
        // self.uart.write(command.as_bytes()).await.unwrap();
    }
}

#[embassy_executor::task]
pub async fn run_mmwave(
    mut mmWave: MmWave<'static>,

) -> ! {
    let test_data: [u8; 2] = ['a' as u8, 'b' as u8];
    mmWave.write(&test_data).await; // Initialize the mmWave to off
    loop {
        // let color = in_channel.receive().await;
        // info!("Received color: {:?} {:?} {:?}", color.r, color.g, color.b);
        // mmWave.write(color).await;

    }
}
