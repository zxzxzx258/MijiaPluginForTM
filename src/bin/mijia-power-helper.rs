use mijia_network_power_dms::{
    config::{self, AppConfig, MiJiaConfig},
    miio,
};
use serde::Serialize;
use std::{collections::HashSet, env, net::Ipv4Addr, path::PathBuf};
use tokio::task::JoinSet;

#[derive(Debug, Serialize)]
struct DeviceReading {
    id: String,
    name: String,
    watts: Option<f64>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProbeOutput {
    devices: Vec<DeviceReading>,
    configured_devices: usize,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImportOutput {
    imported_devices: usize,
    error: Option<String>,
}

enum Command {
    Probe { config: PathBuf },
    ImportText { input: PathBuf, output: PathBuf },
}

fn usage() -> &'static str {
    "Usage: mijia-power-helper --config PATH | --import-text INPUT --output PATH"
}

fn command() -> Result<Command, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [flag, path] if flag == "--config" => Ok(Command::Probe {
            config: PathBuf::from(path),
        }),
        [flag, input, output_flag, output]
            if flag == "--import-text" && output_flag == "--output" =>
        {
            Ok(Command::ImportText {
                input: PathBuf::from(input),
                output: PathBuf::from(output),
            })
        }
        _ => Err(usage().to_string()),
    }
}

#[cfg(unix)]
fn check_config_permissions(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(path)
        .map_err(|err| format!("cannot inspect configuration: {err}"))?
        .permissions()
        .mode()
        & 0o777;
    if mode & 0o077 != 0 {
        return Err("configuration permissions must be 0600 or stricter".to_string());
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_config_permissions(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

async fn probe_device(device: MiJiaConfig) -> DeviceReading {
    let id = device.stable_id();
    let name = device.name.clone();
    match miio::read_power(
        device.ip,
        &device.token,
        &device.model,
        device.power_siid,
        device.power_piid,
    )
    .await
    {
        Ok(reading) => DeviceReading {
            id,
            name,
            watts: Some(reading.watts),
            error: None,
        },
        Err(err) => DeviceReading {
            id,
            name,
            watts: None,
            error: Some(err.to_string()),
        },
    }
}

fn print_output(output: ProbeOutput) {
    // This is the only stdout channel consumed by the DMS daemon.
    println!(
        "{}",
        serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
    );
}

fn print_import_output(output: ImportOutput) {
    println!(
        "{}",
        serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
    );
}

fn extract_ipv4_addresses(text: &str) -> Vec<Ipv4Addr> {
    let mut seen = HashSet::new();
    text.split(|character: char| !character.is_ascii_digit() && character != '.')
        .filter_map(|candidate| candidate.parse::<Ipv4Addr>().ok())
        .filter(|address| seen.insert(*address))
        .collect()
}

fn extract_tokens(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    text.split(|character: char| !character.is_ascii_hexdigit())
        .filter(|candidate| candidate.len() == 32)
        .map(str::to_ascii_lowercase)
        .filter(|token| seen.insert(token.clone()))
        .collect()
}

fn extract_device_pairs(text: &str) -> Result<Vec<(Ipv4Addr, String)>, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let all_tokens = extract_tokens(text);
    let mut matched_tokens = HashSet::new();
    let mut pairs = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let addresses = extract_ipv4_addresses(line);
        if addresses.is_empty() {
            continue;
        }
        if addresses.len() != 1 {
            return Err("an import record contains more than one IPv4 address".to_string());
        }

        let mut candidate_tokens = extract_tokens(line);
        if candidate_tokens.is_empty() {
            // Windows notes commonly put the token on the next line. Stop at
            // another address so an unrelated address can never be paired.
            for next_line in lines.iter().skip(index + 1).take(2) {
                if !extract_ipv4_addresses(next_line).is_empty() {
                    break;
                }
                candidate_tokens = extract_tokens(next_line);
                if !candidate_tokens.is_empty() {
                    break;
                }
            }
        }
        if candidate_tokens.is_empty() {
            continue;
        }
        if candidate_tokens.len() != 1 {
            return Err("an import record contains more than one miIO token".to_string());
        }

        let token = candidate_tokens.remove(0);
        if !matched_tokens.insert(token.clone()) {
            return Err("a miIO token appears in more than one import record".to_string());
        }
        pairs.push((addresses[0], token));
    }

    if pairs.len() != all_tokens.len() {
        return Err("each miIO token must have one adjacent IPv4 address".to_string());
    }
    if pairs.is_empty() {
        return Err("no adjacent IPv4 address and miIO token record was found".to_string());
    }
    Ok(pairs)
}

fn import_text(input: PathBuf, output: PathBuf) -> ImportOutput {
    let bytes = match std::fs::read(&input) {
        Ok(bytes) => bytes,
        Err(error) => {
            return ImportOutput {
                imported_devices: 0,
                error: Some(format!("cannot read import source: {error}")),
            };
        }
    };
    // IP addresses and miIO tokens are ASCII, so this also works for a legacy
    // Windows text file whose Chinese labels use a non-UTF-8 code page.
    let text = String::from_utf8_lossy(&bytes);
    let pairs = match extract_device_pairs(&text) {
        Ok(pairs) => pairs,
        Err(error) => {
            return ImportOutput {
                imported_devices: 0,
                error: Some(error),
            };
        }
    };

    let mijia = pairs
        .into_iter()
        .enumerate()
        .map(|(index, (address, token))| MiJiaConfig {
            id: format!("mijia-{}", index + 1),
            ip: address.into(),
            token,
            name: format!("米家设备 {}", index + 1),
            model: "local.mijia.device".to_string(),
            power_siid: 11,
            power_piid: 2,
        })
        .collect::<Vec<_>>();
    let imported_devices = mijia.len();
    let config = AppConfig {
        mijia,
        ..AppConfig::default()
    };
    match config::save_to(&output, &config) {
        Ok(()) => ImportOutput {
            imported_devices,
            error: None,
        },
        Err(error) => ImportOutput {
            imported_devices: 0,
            error: Some(format!("cannot save configuration: {error}")),
        },
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let command = match command() {
        Ok(command) => command,
        Err(error) => {
            print_output(ProbeOutput {
                devices: Vec::new(),
                configured_devices: 0,
                error: Some(error),
            });
            return;
        }
    };

    if let Command::ImportText { input, output } = command {
        print_import_output(import_text(input, output));
        return;
    }
    let Command::Probe { config: path } = command else {
        unreachable!("import command returned above");
    };

    if let Err(error) = check_config_permissions(&path) {
        print_output(ProbeOutput {
            devices: Vec::new(),
            configured_devices: 0,
            error: Some(error),
        });
        return;
    }

    let config = match config::load_from(&path) {
        Ok(config) => config,
        Err(error) => {
            print_output(ProbeOutput {
                devices: Vec::new(),
                configured_devices: 0,
                error: Some(format!("cannot load configuration: {error}")),
            });
            return;
        }
    };

    let configured_devices = config.mijia.len();
    let mut tasks = JoinSet::new();
    for device in config.mijia {
        tasks.spawn(probe_device(device));
    }

    let mut devices = Vec::with_capacity(configured_devices);
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(reading) => devices.push(reading),
            Err(error) => devices.push(DeviceReading {
                id: "unknown".to_string(),
                name: "米家设备".to_string(),
                watts: None,
                error: Some(format!("probe task failed: {error}")),
            }),
        }
    }
    devices.sort_by(|left, right| left.id.cmp(&right.id));

    print_output(ProbeOutput {
        devices,
        configured_devices,
        error: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_addresses_and_tokens_without_decoding_labels() {
        let text = "192.168.1.10 token=ABCDEF0123456789ABCDEF0123456789\n192.168.1.10";
        assert_eq!(
            extract_ipv4_addresses(text),
            vec![Ipv4Addr::new(192, 168, 1, 10)]
        );
        assert_eq!(
            extract_tokens(text),
            vec!["abcdef0123456789abcdef0123456789".to_string()]
        );
    }

    #[test]
    fn pairs_only_adjacent_ip_and_token_records() {
        let text = "说明地址 10.0.0.1\n\n设备一 192.168.1.10\ntoken=abcdef0123456789abcdef0123456789\n设备二 192.168.1.11\nabcdef0123456789abcdef0123456788";
        assert_eq!(
            extract_device_pairs(text).unwrap(),
            vec![
                (
                    Ipv4Addr::new(192, 168, 1, 10),
                    "abcdef0123456789abcdef0123456789".to_string()
                ),
                (
                    Ipv4Addr::new(192, 168, 1, 11),
                    "abcdef0123456789abcdef0123456788".to_string()
                )
            ]
        );
    }
}
