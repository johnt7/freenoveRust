use defmt::info;
use embassy_time::{Duration,Timer};
use super::SomeSensorState;
use super::SR501_SIGNAL;

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