use embassy_time::{Duration,Timer};

#[embassy_executor::task]
pub async fn run(mut led: esp_hal::gpio::Output<'static>, delay: Duration) -> ! {
    loop {
        led.toggle();
        Timer::after(delay).await;
    }
}
