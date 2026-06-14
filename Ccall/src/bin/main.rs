#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::rmt::Rmt;
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::uart::Config as UartConfig;
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use esp_backtrace as _;
use esp_println::println;
use esp32_s3_ccall::{SerialPort, add_numbers_from_c};
use smart_leds::{RGB8, SmartLedsWrite as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3 -o esp32s3-wroom-1 -o vscode -o esp

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let frequency = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, frequency).expect("failed to initialize RMT");
    let mut ws2812_buf = smart_led_buffer!(1);
    let mut ws2812 = SmartLedsAdapter::new(rmt.channel0, peripherals.GPIO48, &mut ws2812_buf);

    let mut serial = SerialPort::new(peripherals.UART0, UartConfig::default())
        .expect("failed to initialize UART0")
        .with_pins(peripherals.GPIO44, peripherals.GPIO43);

    let colors = [
        RGB8 { r: 32, g: 0, b: 0 },
        RGB8 { r: 0, g: 32, b: 0 },
        RGB8 { r: 0, g: 0, b: 32 },
        RGB8 { r: 0, g: 0, b: 0 },
    ];
    let mut idx = 0usize;

    println!("app started");
    serial.write(b"serial initialized\r\n").unwrap();
    serial.flush().unwrap();

    let sum = add_numbers_from_c(21, 21);
    println!("C FFI example: add_numbers(21, 21) = {}", sum);

    loop {
        // println!("looping...");
        let mut line = [0u8; 64];
        if let Ok(Some(line_len)) = serial.read_line(&mut line) {
            let _ = serial.write(b"Line: ");
            let _ = serial.write(&line[..line_len]);
            let _ = serial.write(b"\r\n");
            let _ = serial.flush();
        }
        ws2812.write([colors[idx]].into_iter()).unwrap();
        idx = (idx + 1) % colors.len();
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
