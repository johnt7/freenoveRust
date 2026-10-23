use embassy_time::{Duration,Timer};

/// A task that runs the heartbeat LED, toggling it at a specified interval.
///
/// # Arguments
/// * `led` - The LED output pin to toggle.
/// * `delay` - The duration to wait between toggles.
/// 
/// # Returns
/// This function never returns; it runs indefinitely.
#[embassy_executor::task]
pub async fn run(mut led: esp_hal::gpio::Output<'static>, delay: Duration) -> ! {
    loop {
        led.toggle();
        Timer::after(delay).await;
    }
}
