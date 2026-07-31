use defmt::info;
use smart_leds::RGB8;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Sender, Receiver};
use heapless::Vec;
// use super::sr501::SR501_SIGNAL;

#[embassy_executor::task]
pub async fn text_handler(
    in_receiver: Receiver<'static, CriticalSectionRawMutex, u8, 100>,
    out_sender: Sender<'static, CriticalSectionRawMutex, u8, 100>,
    neo_sender: Sender<'static, CriticalSectionRawMutex, RGB8, 5>
) -> ! {

    let mut buffer = Vec::<u8, 100>::new(); // Create a heapless buffer with a capacity of 100 bytes

    loop {
        let byte = in_receiver.receive().await;
        // Add the received byte to the buffer
        if buffer.push(byte).is_err() {
            info!("Buffer overflow, clearing buffer");
            buffer.clear(); // Clear the buffer if it overflows
        }

        // Check for a newline character to indicate the end of a message
        if byte == b'\n' {
            if buffer == b"red\r\n" {
                info!("Received 'red' command");
                neo_sender.send(RGB8 { r: 255, g: 0, b: 0 }).await; // Send a green color to the NeoPixel channel
            } else if buffer == b"green\r\n" {
                info!("Received 'green' command");
                neo_sender.send(RGB8 { r: 0, g: 255, b: 0 }).await; // Send a green color to the NeoPixel channel
            } else if buffer == b"blue\r\n" {
                info!("Received 'blue' command");
                neo_sender.send(RGB8 { r: 0, g: 0, b: 255 }).await; // Send a blue color to the NeoPixel channel
            } else {
                info!("Received unknown command: {:?}", buffer);
                neo_sender.send(RGB8 { r: 0, g: 0, b: 0 }).await; // Send a yellow color to the NeoPixel channel
            }
            // Process the complete message here (e.g., send it to another channel)
            for &b in buffer.iter() {
                out_sender.send(b).await;
            }
            buffer.clear(); // Clear the buffer after processing the message
        }
    }
}

