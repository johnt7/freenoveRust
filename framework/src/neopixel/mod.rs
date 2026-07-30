use esp_hal::rmt::Channel;
use ws2812_rmt::{buffer_len, Timing, Ws2812, RGB8};

pub struct NeoPixel {
    // pin: esp_hal::gpio::Output<'static>,
    // rmt: Ws2812,
}

impl NeoPixel {
    pub fn new(pin: esp_hal::gpio::Output<'static>) -> Self {
        // let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80))?;
        // let mut led = Ws2812::<{ buffer_len(1) }>::new(
        //     rmt.channel0,
        //     peripherals.GPIO1,
        //     Timing::WS2812_AT_12_5NS_TICK_ESP32C6,
        //     2, // clk_divider -- must match the constant's own doc comment
        // )?;
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        // // Implement the logic to send color data to the NeoPixel
        // // This is a placeholder implementation
        // let pixels = [RGB8 { r: r, g: g, b: b }; 10];
        // self.dev.write(pixels.iter().cloned()).unwrap();
    }
}