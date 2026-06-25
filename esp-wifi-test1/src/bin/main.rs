#![no_std]
#![no_main]

extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{Config, WifiController};
use panic_rtt_target as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn wifi_connect_task(mut wifi_controller: WifiController<'static>) -> ! {
    const WIFI_SSID: &str = "Enjoy Every Sandwich";
    const WIFI_PASSWORD: &str = "burlingtonarmalite";
    const TARGET_BSSID: [u8; 6] = [0x70, 0x4f, 0x57, 0x93, 0x7c, 0xdf];
    const TARGET_CHANNEL: u8 = 11;

    let station_cfg = StationConfig::default()
        .with_ssid(WIFI_SSID)
        .with_password(WIFI_PASSWORD.into())
        .with_bssid(TARGET_BSSID)
        .with_channel(TARGET_CHANNEL);

    let wifi_cfg = Config::Station(station_cfg);

    wifi_controller
        .set_config(&wifi_cfg)
        .expect("Failed to set Wi-Fi station configuration");

    info!("Starting Wi-Fi connect loop for SSID={} CH={} BSSID={=[u8]:x}", WIFI_SSID, TARGET_CHANNEL, &TARGET_BSSID[..]);

    loop {
        match wifi_controller.connect_async().await {
            Ok(info) => {
                info!(
                    "Connected SSID={} CH={} BSSID={=[u8]:x}",
                    info.ssid.as_str(),
                    info.channel,
                    &info.bssid[..]
                );
                match wifi_controller.wait_for_disconnect_async().await {
                    Ok(disconnect_info) => {
                        info!(
                            "Disconnected from SSID={} reason={:?}",
                            disconnect_info.ssid.as_str(),
                            disconnect_info.reason
                        );
                    }
                    Err(err) => info!("wait_for_disconnect_async failed: {:?}", err),
                }
            }
            Err(err) => info!("Connect failed: {:?}", err),
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}

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

    let (wifi_controller, _interfaces) = esp_radio::wifi::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    spawner.spawn(
        wifi_connect_task(wifi_controller)
        .expect("Failed to create Wi-Fi connect task"),
    );
    spawner.spawn(
        heartbeat(
            esp_hal::gpio::Output::new(peripherals.GPIO2, esp_hal::gpio::Level::Low, Default::default()),
            Duration::from_millis(500)
        )
        .expect("Failed to create heartbeat task"),
    );

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
