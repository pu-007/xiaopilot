use anyhow::{bail, Context, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

#[derive(Deserialize)]
struct Config {
    topic: String,
    commands: HashMap<String, Action>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum Action {
    Shutdown,
    Reboot,
    Sleep,
    Monitor {
        mode: String,
        #[serde(default)]
        default_width: Option<u32>,
    },
    Key {
        keys: Vec<String>,
    },
}

fn get_screen_width() -> Result<u32> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width",
        ])
        .output()
        .context("Failed to run PowerShell for screen width")?;
    let s = String::from_utf8(output.stdout)?;
    s.trim()
        .parse()
        .context("Failed to parse screen width from PowerShell output")
}

fn parse_key(s: &str) -> Result<Key> {
    Ok(match s {
        "Alt" | "AltLeft" => Key::Alt,
        "Ctrl" | "Control" => Key::Control,
        "Shift" => Key::Shift,
        "Meta" | "Win" | "Windows" => Key::Meta,
        "Space" => Key::Space,
        "Escape" | "Esc" => Key::Escape,
        "Enter" | "Return" => Key::Return,
        "Tab" => Key::Tab,
        "Backspace" => Key::Backspace,
        "Delete" | "Del" => Key::Delete,
        "Insert" | "Ins" => Key::Insert,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Up" | "UpArrow" => Key::UpArrow,
        "Down" | "DownArrow" => Key::DownArrow,
        "Left" | "LeftArrow" => Key::LeftArrow,
        "Right" | "RightArrow" => Key::RightArrow,
        "CapsLock" => Key::CapsLock,
        "NumLock" => Key::Numlock,
        "PrintScreen" | "PrtSc" => Key::PrintScr,
        "Pause" => Key::Pause,
        s if s.starts_with('F') && s.len() >= 2 => {
            let n: u8 = s[1..]
                .parse()
                .context(format!("Invalid function key: {}", s))?;
            match n {
                1 => Key::F1,
                2 => Key::F2,
                3 => Key::F3,
                4 => Key::F4,
                5 => Key::F5,
                6 => Key::F6,
                7 => Key::F7,
                8 => Key::F8,
                9 => Key::F9,
                10 => Key::F10,
                11 => Key::F11,
                12 => Key::F12,
                _ => bail!("Unknown function key: {}", s),
            }
        }
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
        _ => bail!(
            "Unknown key: '{}'. Use Alt, Ctrl, Shift, Meta/Win, Space, Escape, Enter, Tab, \
             Backspace, Delete, Insert, Home, End, PageUp, PageDown, arrow keys, \
             CapsLock, NumLock, PrintScreen, Pause, F1-F12, or a single character.",
            s
        ),
    })
}

fn execute_keys(keys: &[String]) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("Failed to create Enigo")?;
    let parsed: Vec<Key> = keys.iter().map(|k| parse_key(k)).collect::<Result<Vec<_>>>()?;

    match parsed.len() {
        0 => {}
        1 => {
            enigo.key(parsed[0], Direction::Click)?;
        }
        _ => {
            let (modifiers, target) = parsed.split_at(parsed.len() - 1);
            let target = target[0];

            for &m in modifiers {
                enigo.key(m, Direction::Press)?;
            }
            enigo.key(target, Direction::Click)?;
            for &m in modifiers.iter().rev() {
                enigo.key(m, Direction::Release)?;
            }
        }
    }
    Ok(())
}

fn execute(action: &Action) -> Result<()> {
    match action {
        Action::Shutdown => {
            Command::new("shutdown")
                .args(["-s", "-t", "0"])
                .spawn()
                .context("Failed to run shutdown command")?;
        }
        Action::Reboot => {
            Command::new("shutdown")
                .args(["-r", "-t", "0"])
                .spawn()
                .context("Failed to run reboot command")?;
        }
        Action::Sleep => {
            Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0", "1", "0"])
                .spawn()
                .context("Failed to run sleep command")?;
        }
        Action::Monitor { mode, default_width } => match mode.as_str() {
            "pc" => {
                Command::new("displayswitch.exe").arg("1").spawn()?;
            }
            "copy" => {
                Command::new("displayswitch.exe").arg("2").spawn()?;
            }
            "extend" => {
                Command::new("displayswitch.exe").arg("3").spawn()?;
            }
            "tv" => {
                Command::new("displayswitch.exe").arg("4").spawn()?;
            }
            "switch" => {
                let dw = default_width.unwrap_or(2560);
                let width = get_screen_width().unwrap_or(0);
                if width == dw {
                    Command::new("displayswitch.exe").arg("4").spawn()?;
                } else {
                    Command::new("displayswitch.exe").arg("1").spawn()?;
                }
            }
            _ => bail!("Unknown monitor mode: {}", mode),
        },
        Action::Key { keys } => {
            execute_keys(keys)?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config_path = std::env::current_dir()?.join("win.yml");
    if !config_path.exists() {
        bail!("win.yml not found. Copy win.yml.example and edit it.");
    }

    let content = std::fs::read_to_string(&config_path).context("Failed to read win.yml")?;
    let config: Config = serde_yaml::from_str(&content).context("Failed to parse win.yml")?;

    let client_id =
        std::env::var("BAFA_ID").context("BAFA_ID not set in .env or environment")?;

    let mut mqtt_opts = MqttOptions::new(&client_id, "bemfa.com", 9501);
    mqtt_opts.set_keep_alive(Duration::from_secs(60));
    mqtt_opts.set_credentials("userName", "passwd");

    let (client, mut eventloop) = AsyncClient::new(mqtt_opts, 10);

    client
        .subscribe(&config.topic, QoS::AtMostOnce)
        .await
        .context("Failed to subscribe to MQTT topic")?;

    eprintln!(
        "[xiaopilot-win] Connected to Bafa, {} commands registered on topic '{}'",
        config.commands.len(),
        config.topic,
    );

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let trigger = String::from_utf8_lossy(&msg.payload);
                let trigger = trigger.trim();
                eprintln!("[xiaopilot-win] Received trigger: {}", trigger);

                match config.commands.get(trigger) {
                    Some(action) => {
                        eprintln!("[xiaopilot-win] Executing: {:?}", trigger);
                        if let Err(e) = execute(action) {
                            eprintln!("[xiaopilot-win] Command failed: {:#}", e);
                        }
                    }
                    None => {
                        eprintln!("[xiaopilot-win] Unknown trigger: {}", trigger);
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[xiaopilot-win] MQTT error: {}, reconnecting...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
