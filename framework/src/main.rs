#![no_std]
#![no_main]

extern crate alloc;

// use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{StackResources};
use embassy_sync::channel::Channel;
use embassy_time::{Duration};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use esp_hal::clock::CpuClock;

use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use panic_rtt_target as _;
use static_cell::StaticCell;

use esp_hal::rmt::TxChannelCreator;
use esp_hal::{
    gpio::{Level},
    rmt::{Rmt, TxChannelConfig},
    uart::{Config as UartConfig, Uart},
};
use smart_leds::RGB8;

mod heartbeat;
mod wifi;
mod internal;
mod sr501;
mod ws218;
mod mmwave;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

static NET_STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static WIFI_IN_CHANNEL: Channel<CriticalSectionRawMutex, u8, 100> = Channel::new();
static WIFI_OUT_CHANNEL: Channel<CriticalSectionRawMutex, u8, 100> = Channel::new();
static NEO_PIXEL_CHANNEL: Channel<CriticalSectionRawMutex, RGB8, 5> = Channel::new();

// #[embassy_executor::task]
// async fn uart_rx_task(mut uart: Uart<'static, esp_hal::Async>) -> ! {
//     let mut byte = [0u8; 1];

//     loop {
//         uart.read_exact_async(&mut byte)
//             .await
//             .expect("UART read failed");
//         info!("UART RX byte: {}", byte[0]);
//     }
// }


#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // pin for the SR501 sensor
    let sr501pin = esp_hal::gpio::Input::new(peripherals.GPIO38, esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::None));

    // Set up the wifi handling 
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    let (wifi_controller, interfaces) = esp_radio::wifi::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");
    let (stack, mut runner) = embassy_net::new(
        interfaces.station,
        embassy_net::Config::dhcpv4(Default::default()),
        NET_STACK_RESOURCES.init(StackResources::new()),
        0x0123_4567_89ab_cdef,
    );

    // Set up the NeoPixel
    let neopixel_pin = esp_hal::gpio::Output::new(peripherals.GPIO48, esp_hal::gpio::Level::Low, Default::default());
    let rmt= Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    let tx_cfg = TxChannelConfig::default()
    .with_clk_divider(1)
    .with_idle_output(true)
    .with_idle_output_level(Level::Low);
    let channel = rmt.channel0.configure_tx(&tx_cfg).unwrap().with_pin(neopixel_pin);
    let neop = ws218::NeoPixel::new(channel);

    // set up the mmWave
    let mmwave_pin = esp_hal::gpio::Input::new(peripherals.GPIO47, esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::None));
    let mmwave_uart = Uart::new(
        peripherals.UART0,
        UartConfig::default(),
    )
    .expect("failed to initialize UART0")
    .with_tx(peripherals.GPIO43)
    .with_rx(peripherals.GPIO44)
    .into_async();
    let mmwave = mmwave::MmWave::new(mmwave_pin, mmwave_uart);



    // Run the tasks
    // heartbeat LED task
    spawner.spawn(
        heartbeat::run(
            esp_hal::gpio::Output::new(peripherals.GPIO2, esp_hal::gpio::Level::Low, Default::default()),
            Duration::from_millis(500)
        )
        .expect("Failed to create heartbeat task"),
    );
    // NeoPixel task
    spawner.spawn(
        ws218::run_ws218(
            neop,
            NEO_PIXEL_CHANNEL.receiver()
        )
        .expect("Failed to create NeoPixel task"),
    );
    // wifi handling
    spawner.spawn(
        wifi::wifi_handling(wifi_controller, stack)
        .expect("Failed to create Wi-Fi connect task"),
    );
    // set up socket handler to pass data to channels
    spawner.spawn(
        wifi::socket_setup(stack, WIFI_IN_CHANNEL.sender(), WIFI_OUT_CHANNEL.receiver())
        .expect("Failed to create socket"),
    );
    // Sample handler that reads from the input channel and writes to the output channel
    spawner.spawn(
        internal::text_handler(WIFI_IN_CHANNEL.receiver(), WIFI_OUT_CHANNEL.sender(), NEO_PIXEL_CHANNEL.sender())
        .expect("Failed to create internal handler"),
    );
    // SR501 sensor task
    spawner.spawn(
        sr501::sr501_task(sr501pin, Duration::from_millis(5000))
        .expect("Failed to create SR501 handler"),
    );
    // mmWave UART task
    spawner.spawn(
        mmwave::uart_rx_task(mmwave)
        .expect("Failed to create UART token")
    );

    runner.run().await;
}
