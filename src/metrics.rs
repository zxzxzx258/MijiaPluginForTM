use std::{collections::HashMap, fs, io, path::Path, time::Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NetworkRate {
    pub rx_bytes_per_second: f64,
    pub tx_bytes_per_second: f64,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkSnapshot {
    pub interface: String,
    pub rate: NetworkRate,
}

#[derive(Debug, Clone, Default)]
pub struct CounterSnapshot {
    counters: HashMap<String, (u64, u64)>,
    at: Option<Instant>,
}

impl CounterSnapshot {
    pub fn sample(&mut self, interface_filter: Option<&str>) -> io::Result<NetworkSnapshot> {
        let now = Instant::now();
        let current = read_counters()?;
        let selected = interface_filter
            .and_then(|name| current.get(name).map(|c| (name.to_string(), *c)))
            .or_else(|| {
                automatic_interface().and_then(|name| current.get(&name).map(|c| (name, *c)))
            })
            .or_else(|| {
                active_physical_interfaces()
                    .into_iter()
                    .filter_map(|name| current.get(&name).map(|c| (name, *c)))
                    .max_by_key(|(_, c)| c.0 + c.1)
            });
        let (interface, (rx, tx)) = selected.unwrap_or_else(|| ("-".into(), (0, 0)));
        let rate = match (self.at, self.counters.get(&interface)) {
            (Some(previous_at), Some((old_rx, old_tx))) => {
                let seconds = now.duration_since(previous_at).as_secs_f64().max(0.001);
                NetworkRate {
                    rx_bytes_per_second: rx.saturating_sub(*old_rx) as f64 / seconds,
                    tx_bytes_per_second: tx.saturating_sub(*old_tx) as f64 / seconds,
                }
            }
            _ => NetworkRate::default(),
        };
        self.counters = current;
        self.at = Some(now);
        Ok(NetworkSnapshot { interface, rate })
    }
}

pub fn active_physical_interfaces() -> Vec<String> {
    let mut interfaces = fs::read_dir("/sys/class/net")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().into_string().ok()?;
            let state = fs::read_to_string(path.join("operstate")).ok()?;
            if name != "lo" && state.trim() == "up" && is_physical_path(&path) {
                Some(name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    interfaces.sort();
    interfaces
}

fn is_physical_interface(interface: &str) -> bool {
    is_physical_path(&Path::new("/sys/class/net").join(interface))
}

fn is_physical_path(path: &Path) -> bool {
    path.join("device").exists()
}

pub fn default_route_interface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;
    parse_default_route(&routes)
}

pub fn automatic_interface() -> Option<String> {
    default_route_interface()
        .filter(|name| is_physical_interface(name))
        .or_else(|| active_physical_interfaces().into_iter().next())
}

fn parse_default_route(routes: &str) -> Option<String> {
    routes
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < 8 || fields[1] != "00000000" || fields[7] != "00000000" {
                return None;
            }
            let flags = u16::from_str_radix(fields[3], 16).ok()?;
            if flags & 1 == 0 {
                return None;
            }
            let metric = fields[6].parse::<u32>().ok()?;
            Some((metric, fields[0].to_string()))
        })
        .min_by_key(|(metric, _)| *metric)
        .map(|(_, interface)| interface)
}

fn read_counters() -> io::Result<HashMap<String, (u64, u64)>> {
    let text = fs::read_to_string("/proc/net/dev")?;
    Ok(text
        .lines()
        .skip(2)
        .filter_map(|line| {
            let (name, values) = line.split_once(':')?;
            let numbers: Vec<u64> = values
                .split_whitespace()
                .filter_map(|value| value.parse().ok())
                .collect();
            Some((
                name.trim().to_string(),
                (*numbers.first()?, *numbers.get(8)?),
            ))
        })
        .collect())
}

pub fn format_rate(bytes_per_second: f64) -> String {
    let units = ["B/s", "KiB/s", "MiB/s", "GiB/s"];
    let mut value = bytes_per_second.max(0.0);
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.0} {}", units[unit])
}

pub fn format_rate_compact(bytes_per_second: f64) -> String {
    format_rate(bytes_per_second).replace(" ", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats_rates() {
        assert_eq!(format_rate(1536.0), "2 KiB/s");
        assert_eq!(format_rate(0.0), "0 B/s");
        assert_eq!(format_rate_compact(1024.0), "1KiB/s");
    }

    #[test]
    fn selects_lowest_metric_default_route() {
        let routes = "Iface Destination Gateway Flags RefCnt Use Metric Mask\n\
                      wlan0 00000000 0102A8C0 0003 0 0 600 00000000\n\
                      enp14s0 00000000 0102A8C0 0003 0 0 100 00000000\n";
        assert_eq!(parse_default_route(routes).as_deref(), Some("enp14s0"));
    }
}
