#![no_std]
#![no_main]

extern crate alloc;

// use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use panic_rtt_target as _;
use static_cell::StaticCell;

mod heartbeat;
mod wifi;
mod internal;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

static NET_STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static WIFI_IN_CHANNEL: Channel<CriticalSectionRawMutex, u8, 100> = Channel::new();
static WIFI_OUT_CHANNEL: Channel<CriticalSectionRawMutex, u8, 100> = Channel::new();

#[embassy_executor::task]
async fn heartbeat(mut led: esp_hal::gpio::Output<'static>, delay: Duration) -> ! {
    loop {
        led.toggle();
        Timer::after(delay).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    // Required for Wi-Fi firmware tasks.
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let (wifi_controller, interfaces) = esp_radio::wifi::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    // let in_sender = WIFI_IN_CHANNEL.sender();
    // let in_receiver = WIFI_IN_CHANNEL.receiver();
    // let out_sender = WIFI_OUT_CHANNEL.sender();
    // let out_receiver = WIFI_OUT_CHANNEL.receiver();

    let (stack, mut runner) = embassy_net::new(
        interfaces.station,
        embassy_net::Config::dhcpv4(Default::default()),
        NET_STACK_RESOURCES.init(StackResources::new()),
        0x0123_4567_89ab_cdef,
    );

    spawner.spawn(
        heartbeat::run(
            esp_hal::gpio::Output::new(peripherals.GPIO2, esp_hal::gpio::Level::Low, Default::default()),
            Duration::from_millis(500)
        )
        .expect("Failed to create heartbeat task"),
    );
    spawner.spawn(
        // wifi::wifi_setup(wifi_controller, stack)
        wifi::wifi_handling(wifi_controller, stack)
        .expect("Failed to create Wi-Fi connect task"),
    );
    spawner.spawn(
        wifi::socket_setup(stack, WIFI_IN_CHANNEL.sender(), WIFI_OUT_CHANNEL.receiver())
        .expect("Failed to create socket"),
    );
    spawner.spawn(
        internal::text_handler(WIFI_IN_CHANNEL.receiver(), WIFI_OUT_CHANNEL.sender())
        .expect("Failed to create internal handler"),
    );

    runner.run().await;
}
