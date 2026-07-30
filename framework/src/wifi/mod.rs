use defmt::info;
use embedded_io_async::Write;

use embassy_net::{Ipv4Address, Stack};
use embassy_net::tcp::TcpSocket;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Sender, Receiver};
use embassy_futures::select::{select, Either};

use embassy_time::{Duration, Timer};
use embassy_time::with_timeout;

use esp_radio::wifi::{Config, WifiController};
use esp_radio::wifi::sta::StationConfig;

static WIFI_RETRY_INTERVAL_SECONDS: u64 = 5;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

const TARGET_IP_OCTETS: [u8; 4] = [192, 168, 0, 2];
const TARGET_IP: Ipv4Address = Ipv4Address::new(
    TARGET_IP_OCTETS[0],
    TARGET_IP_OCTETS[1],
    TARGET_IP_OCTETS[2],
    TARGET_IP_OCTETS[3],
);

#[embassy_executor::task]
pub async fn wifi_handling(mut wifi_controller: WifiController<'static>, stack: Stack<'static>) -> ! {
    let station_cfg = StationConfig::default()
        .with_ssid(WIFI_SSID)
        .with_password(WIFI_PASSWORD.into());

    let wifi_cfg = Config::Station(station_cfg);

    wifi_controller
        .set_config(&wifi_cfg)
        .expect("Failed to set Wi-Fi station configuration");

    info!("xStarting Wi-Fi connect loop for SSID={}", WIFI_SSID);

    'outer: loop {
        // wait until the wifi is connected.
        info!("xWi-Fi connect loop iteration");
        loop {
            if !wifi_controller.is_connected() {
                match wifi_controller.connect_async().await {
                    Ok(info) => {
                        info!(
                            "xConnected SSID={} CH={} BSSID={=[u8]:x}",
                            info.ssid.as_str(),
                            info.channel,
                            &info.bssid[..]
                        );
                        break;
                    }
                    Err(err) => {
                        info!("xConnect failed: {:?}, wait 5 seconds", err);
                        Timer::after(Duration::from_secs(5)).await;
                        // continue;
                    }
                }
            }
        }

        loop {
            if !stack.is_config_up() {
                if with_timeout(Duration::from_secs(5), stack.wait_config_up())
                    .await
                    .is_err()
                {
                    info!("DHCP timeout after 5s, reconnecting Wi-Fi");
                    let _ = wifi_controller.disconnect_async().await;
                    continue 'outer;
                 }

            } else {
                info!("xDHCP IPv4 configuration acquired");
                break;
            }
        }

        loop {
            if let Some(config) = stack.config_v4() {
                info!(
                    "xLocal IPv4={} gateway={:?} dns={:?}",
                    config.address,
                    config.gateway,
                    config.dns_servers
                );
                break;
            }
        }

        loop {
             if let None = stack.config_v4() {
                info!(
                    "No IPv4 configuration available",
                );
                break;
            }
            Timer::after(Duration::from_secs(5)).await;
        }
        Timer::after(Duration::from_secs(WIFI_RETRY_INTERVAL_SECONDS)).await;
    }
}

#[embassy_executor::task]
pub async fn socket_setup(stack: Stack<'static>, in_sender: Sender<'static, CriticalSectionRawMutex, u8, 100>, out_receiver: Receiver<'static, CriticalSectionRawMutex, u8, 100>) -> ! {
    const TARGET_PORT: u16 = 5000;
    // const PAYLOAD: &[u8] = b"hello from esp32s3\n";
    const TCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let mut inbuf = [0u8; 1024];
    info!("socket setup:now get socket");
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

    loop {
        info!("socket setup: Waiting for DHCP IPv4 configuration");
        loop {
            if let Some(config) = stack.config_v4() {
                info!(
                    "socket setup: Local IPv4={} gateway={:?} dns={:?}",
                    config.address,
                    config.gateway,
                    config.dns_servers
                );
                break;
            }
            info!("Socket setup: No IPv4 configuration available, wait 5 seconds");
            Timer::after(Duration::from_secs(5)).await;
        }

        info!("socket setup:now get socket");
        match with_timeout(
            TCP_CONNECT_TIMEOUT,
            socket.connect((TARGET_IP, TARGET_PORT)),
        )
        .await
        {
            Err(_) => {
                info!(
                    "socket setup: TCP connect timeout after {}s to {}.{}.{}.{}:{}",
                    TCP_CONNECT_TIMEOUT.as_secs(),
                    TARGET_IP_OCTETS[0],
                    TARGET_IP_OCTETS[1],
                    TARGET_IP_OCTETS[2],
                    TARGET_IP_OCTETS[3],
                    TARGET_PORT
                );
            }
            Ok(Ok(())) => {
                info!(
                    "socket setup: TCP connected to {}.{}.{}.{}:{}",
                    TARGET_IP_OCTETS[0],
                    TARGET_IP_OCTETS[1],
                    TARGET_IP_OCTETS[2],
                    TARGET_IP_OCTETS[3],
                    TARGET_PORT
                );

                // let mut delay_start = Instant::now();
                loop {
                    match select(socket.read(&mut inbuf), out_receiver.receive()).await {
                        Either::First(Ok(n)) => {
                            // push inbuf into in_sender
                            for &byte in &inbuf[..n] {
                                in_sender.send(byte).await;
                            }
                            // info!("Received {} bytes: {:?}", n, &inbuf[..n]);
                        }
                        Either::First(Err(e)) => {
                            info!("socket setup: TCP read failed: {:?}", e);
                            break;
                        }
                        Either::Second(c) => {
                            socket.write_all(&[c]).await.expect("Failed to write to socket");
                        }
                    }
                }
            }
            Ok(Err(err)) => info!("socket setup:TCP connect failed: {:?}", err),
        }

        Timer::after(Duration::from_secs(5)).await;
    }

}