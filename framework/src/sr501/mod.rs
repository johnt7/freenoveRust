// use defmt::info;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration,Timer};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SomeSensorState {
  On,
  Off,
}
pub static SR501_SIGNAL: Signal<CriticalSectionRawMutex, SomeSensorState> = Signal::new();


/// A task that monitors the SR501 motion sensor and signals state changes.
///
/// # Arguments
/// * `sr501pin` - The input pin connected to the SR501 sensor.
/// * `delay` - The duration to wait between sensor checks.
///
/// # Returns
/// This function never returns; it runs indefinitely.
/// 
/// It continuously checks the state of the SR501 sensor and signals any changes via `SR501_SIGNAL`.

#[embassy_executor::task]
pub async fn sr501_task(
    sr501pin: esp_hal::gpio::Input<'static>,
    delay: Duration,
) -> ! {
    let mut last: bool = false;
    loop {
        let state = if sr501pin.is_high() {
            // info!("motion on detected");
            SomeSensorState::On

        } else {
            // info!("motion off detected");
            SomeSensorState::Off
        };

        if (last && state == SomeSensorState::Off) || (!last && state == SomeSensorState::On) {
            last = !last;
            SR501_SIGNAL.signal(state);
        } 
        Timer::after(delay).await;
    }
}