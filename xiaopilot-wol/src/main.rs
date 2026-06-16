use anyhow::{bail, Context, Result};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use std::net::UdpSocket;
use std::time::Duration;

#[derive(Deserialize)]
struct Config {
    topic: String,
    trigger: String,
    mac: String,
    #[serde(default = "default_broadcast")]
    broadcast: String,
}

fn default_broadcast() -> String {
    "255.255.255.255".into()
}

fn send_wol(mac: &str, broadcast: &str) -> Result<()> {
    let mac_bytes: Vec<u8> = mac
        .split(':')
        .map(|s| u8::from_str_radix(s, 16))
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Invalid MAC address format")?;

    if mac_bytes.len() != 6 {
        bail!("MAC address must have exactly 6 octets");
    }

    let mut packet = vec![0xFFu8; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&mac_bytes);
    }

    let addr = format!("{}:9", broadcast);
    let socket = UdpSocket::bind("0.0.0.0:0").context("Failed to bind UDP socket")?;
    socket
        .set_broadcast(true)
        .context("Failed to set broadcast")?;
    socket
        .send_to(&packet, &addr)
        .context("Failed to send WOL packet")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config_path = std::env::current_dir()?.join("wol.yml");
    if !config_path.exists() {
        bail!("wol.yml not found. Copy wol.yml.example and edit it.");
    }

    let content = std::fs::read_to_string(&config_path).context("Failed to read wol.yml")?;
    let config: Config = serde_yaml::from_str(&content).context("Failed to parse wol.yml")?;

    let client_id =
        std::env::var("BAFA_ID").context("BAFA_ID not set in .env or environment")?;
    let user = std::env::var("BAFA_USER").unwrap_or_else(|_| "userName".into());
    let pass = std::env::var("BAFA_PASS").unwrap_or_else(|_| "passwd".into());

    let mut mqtt_opts = MqttOptions::new(&client_id, "bemfa.com", 9501);
    mqtt_opts.set_keep_alive(Duration::from_secs(60));
    mqtt_opts.set_credentials(user, pass);

    let (client, mut eventloop) = AsyncClient::new(mqtt_opts, 10);

    client
        .subscribe(&config.topic, QoS::AtMostOnce)
        .await
        .context("Failed to subscribe to MQTT topic")?;

    eprintln!(
        "[xiaopilot-wol] Connected to Bafa, listening for '{}' on topic '{}', broadcast {}",
        config.trigger, config.topic, config.broadcast
    );

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let trigger = String::from_utf8_lossy(&msg.payload);
                let trigger = trigger.trim();
                eprintln!("[xiaopilot-wol] Received trigger: {}", trigger);

                if trigger == config.trigger {
                    eprintln!("[xiaopilot-wol] Sending WOL to {} via {}", config.mac, config.broadcast);
                    if let Err(e) = send_wol(&config.mac, &config.broadcast) {
                        eprintln!("[xiaopilot-wol] WOL failed: {:#}", e);
                    } else {
                        eprintln!("[xiaopilot-wol] WOL packet sent");
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[xiaopilot-wol] MQTT error: {}, reconnecting...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
