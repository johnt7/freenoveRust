use defmt::info;
use embedded_io_async::Write;

use embassy_net::{Ipv4Address, Stack};
use embassy_net::tcp::TcpSocket;

use embassy_time::{Duration, Timer, Instant};
use embassy_time::with_timeout;

use esp_radio::wifi::{Config, WifiController};
use esp_radio::wifi::sta::StationConfig;

#[embassy_executor::task]
pub async fn wifi_setup(mut wifi_controller: WifiController<'static>, stack: Stack<'static>) -> ! {
    const WIFI_SSID: &str = env!("WIFI_SSID");
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
    const TARGET_BSSID: [u8; 6] = [0x70, 0x4f, 0x57, 0x93, 0x7c, 0xdf];
    const TARGET_CHANNEL: u8 = 11;
    const TARGET_IP_OCTETS: [u8; 4] = [192, 168, 0, 250];
    const TARGET_IP: Ipv4Address = Ipv4Address::new(
        TARGET_IP_OCTETS[0],
        TARGET_IP_OCTETS[1],
        TARGET_IP_OCTETS[2],
        TARGET_IP_OCTETS[3],
    );
    const TARGET_PORT: u16 = 5000;
    const PAYLOAD: &[u8] = b"hello from esp32s3\n";
    const TCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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
        if !wifi_controller.is_connected() {
            match wifi_controller.connect_async().await {
                Ok(info) => {
                    info!(
                        "Connected SSID={} CH={} BSSID={=[u8]:x}",
                        info.ssid.as_str(),
                        info.channel,
                        &info.bssid[..]
                    );
                    break;
                }
                Err(err) => {
                    info!("Connect failed: {:?}, wait 5 seconds", err);
                    Timer::after(Duration::from_secs(5)).await;
                    // continue;
                }
            }
        }
    }

    loop {
        if !stack.is_config_up() {
            info!("Waiting for DHCP IPv4 configuration");
            if with_timeout(Duration::from_secs(20), stack.wait_config_up())
                .await
                .is_err()
            {
                info!("DHCP timeout after 20s, reconnecting Wi-Fi");
                let _ = wifi_controller.disconnect_async().await;
                Timer::after(Duration::from_secs(2)).await;
                // loop{};
            }

        } else {
            info!("DHCP IPv4 configuration acquired");
            break;
        }
    }

    loop {
        if let Some(config) = stack.config_v4() {
            info!(
                "Local IPv4={} gateway={:?} dns={:?}",
                config.address,
                config.gateway,
                config.dns_servers
            );
            break;
        }
    }

    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let mut inbuf = [0u8; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

    loop {

        match with_timeout(
            TCP_CONNECT_TIMEOUT,
            socket.connect((TARGET_IP, TARGET_PORT)),
        )
        .await
        {
            Err(_) => {
                info!(
                    "TCP connect timeout after {}s to {}.{}.{}.{}:{}",
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
                    "TCP connected to {}.{}.{}.{}:{}",
                    TARGET_IP_OCTETS[0],
                    TARGET_IP_OCTETS[1],
                    TARGET_IP_OCTETS[2],
                    TARGET_IP_OCTETS[3],
                    TARGET_PORT
                );

                let mut delay_start = Instant::now();
                loop {
                    if delay_start.elapsed() > Duration::from_secs(2) {
                        if let Err(err) = socket.write_all(PAYLOAD).await {
                            info!("TCP send failed: {:?}", err);
                            break;
                        } else {
                            info!("Sent {} bytes", PAYLOAD.len());
                        }
                        delay_start = Instant::now();
                    }

                    match with_timeout(Duration::from_millis(1), socket.read(&mut inbuf)).await {
                        Ok(Ok(n)) => {
                            info!("Received {} bytes: {:?}", n, &inbuf[..n]);
                            if let Err(err) = socket.write_all(&inbuf[..n]).await {
                                info!("TCP send failed: {:?}", err);
                                break;
                            } else {
                                info!("Sent {} bytes", &inbuf[..n].len());
                            }
                        }
                        Ok(Err(e)) => {
                            info!("TCP read failed: {:?}", e);
                            break;
                        }
                        Err(_) => {
                            // info!("TCP read timeout");
                        }
                    }

                    // if let Err(err) = socket.flush().await {
                    //     info!("TCP flush failed: {:?}", err);
                    // }

                    Timer::after(Duration::from_secs(2)).await;
                }
            }
            Ok(Err(err)) => info!("TCP connect failed: {:?}", err),
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}

