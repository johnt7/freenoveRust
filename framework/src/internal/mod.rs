use defmt::info;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Sender, Receiver};
use heapless::Vec;
use super::SR501_SIGNAL;

#[embassy_executor::task]
pub async fn text_handler(
    in_receiver: Receiver<'static, CriticalSectionRawMutex, u8, 100>,
    out_sender: Sender<'static, CriticalSectionRawMutex, u8, 100>,
) -> ! {

    let mut buffer = Vec::<u8, 100>::new(); // Create a heapless buffer with a capacity of 100 bytes

    loop {
        // TODO - watch the 501
        let byte = in_receiver.receive().await;
        // Add the received byte to the buffer
        if buffer.push(byte).is_err() {
            info!("Buffer overflow, clearing buffer");
            buffer.clear(); // Clear the buffer if it overflows
        }

        // Check for a newline character to indicate the end of a message
        if byte == b'\n' {
            // Process the complete message here (e.g., send it to another channel)
            for &b in buffer.iter() {
                out_sender.send(b).await;
            }
            buffer.clear(); // Clear the buffer after processing the message
        }
    }
}
