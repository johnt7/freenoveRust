
use defmt::info;
use esp_hal::rmt::{Channel, PulseCode, Tx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::gpio::Level;
use esp_hal::Blocking;
use smart_leds::RGB8;
use embassy_time::{Duration, Timer};
use embassy_sync::channel::{Receiver};

pub struct NeoPixel<'ch> {
    channel: Option<Channel<'ch, Blocking, Tx>>,
}

impl<'ch> NeoPixel<'ch> {
    pub fn new(channel: Channel<'ch, Blocking, Tx>) -> Self {
        Self { channel: Some(channel) }
    }

    pub async fn write(&mut self, pixels: RGB8) {
        let mut pulses: [PulseCode; NEO_PULSE_LENGTH] = [PulseCode::end_marker(); NEO_PULSE_LENGTH];
        pixel_to_pulses(pixels, &mut pulses);

        // transmit() consumes the channel; wait() returns it
        let ch = self.channel.take().expect("channel unavailable");
        let tx = ch.transmit(&pulses).unwrap();
        self.channel = Some(tx.wait().unwrap());

        // Reset time > 50 µs
        Timer::after(Duration::from_micros(80)).await;
    }
}

/// A task that controls WS2812 NeoPixels based on incoming color data.
/// # Arguments
/// * `neopixel` - The NeoPixel instance to control the WS2812 LEDs.
/// * `in_channel` - The channel receiver for incoming color data.
///
/// # Returns
/// This function never returns; it runs indefinitely.
///
/// It continuously waits for incoming color data and updates the WS2812 LEDs accordingly.

#[embassy_executor::task]
pub async fn run_ws218(
    mut neopixel: NeoPixel<'static>,
    in_channel: Receiver<'static, CriticalSectionRawMutex, RGB8, 5>

) -> ! {
    neopixel.write(RGB8 { r: 0, g: 0, b: 0 }).await; // Initialize the NeoPixel to off
    loop {
        let color = in_channel.receive().await;
        info!("Received color: {:?} {:?} {:?}", color.r, color.g, color.b);
        neopixel.write(color).await;
    }
}

/// WS2812 timing (ns)
const T0H: u32 = 350;
const T0L: u32 = 800;
const T1H: u32 = 700;
const T1L: u32 = 600;
const MHZ: u32 = 80;
const NEO_PULSE_LENGTH: usize = 25; // 8 bits per color * 3 colors (RGB) + 1 end marker

fn ns_to_ticks(ns: u32) -> u16 {
    ((ns * MHZ) / 1000) as u16
}

/// Convert one bit into a PulseCode
fn bit_to_pulse(bit: bool) -> PulseCode {
    let (h, l) = if bit {
        (ns_to_ticks(T1H), ns_to_ticks(T1L))
    } else {
        (ns_to_ticks(T0H), ns_to_ticks(T0L))
    };
    PulseCode::new(Level::High, h, Level::Low, l)
}

fn pixel_to_pulses(color: RGB8, out: &mut [PulseCode; NEO_PULSE_LENGTH]) {
    let grb = [color.g, color.r, color.b];
    out[NEO_PULSE_LENGTH - 1] = PulseCode::end_marker();

    let mut idx = 0;
    for byte in grb {
        for bit in (0..8).rev() {
            let is_one = (byte >> bit) & 1 == 1;
            out[idx] = bit_to_pulse(is_one);
            idx += 1;
        }
    }
}
