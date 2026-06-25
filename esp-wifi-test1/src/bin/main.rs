#![no_std]
#![no_main]

extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::time::Duration as HalDuration;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::WifiController;
use esp_radio::wifi::scan::{ScanConfig, ScanTypeConfig};
use panic_rtt_target as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn wifi_scan_task(mut wifi_controller: WifiController<'static>, scan_cfg: ScanConfig) -> ! {
    info!("Starting Wi-Fi scan loop");

    loop {
        match wifi_controller.scan_async(&scan_cfg).await {
            Ok(results) => {
                info!("Found {} networks", results.len());
                for ap in &results {
                    info!(
                        "SSID={} RSSI={}dBm CH={} BSSID={=[u8]:x}",
                        ap.ssid.as_str(),
                        ap.signal_strength,
                        ap.channel,
                        &ap.bssid[..]
                    );
                }
            }
            Err(err) => info!("Scan failed: {:?}", err),
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

    let scan_cfg = ScanConfig::default()
        .with_show_hidden(true)
        .with_scan_type(ScanTypeConfig::Active {
            min: HalDuration::from_millis(50),
            max: HalDuration::from_millis(120),
        });

    spawner.spawn(
        wifi_scan_task(wifi_controller, scan_cfg)
        .expect("Failed to create Wi-Fi scan task"),
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
