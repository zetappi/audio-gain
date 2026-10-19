use std::process::Command;

#[derive(Clone, Debug)]
pub struct Sink {
    pub name: String,
    pub description: String,
}

pub fn list_sinks() -> Vec<Sink> {
    let out = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C")
        .args(["list", "sinks"])
        .output()
        .ok();
    let text = out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    parse_sinks(&text)
}

fn parse_sinks(text: &str) -> Vec<Sink> {
    let mut sinks = Vec::new();
    let mut cur_name = None;
    let mut cur_desc = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Name:") {
            cur_name = Some(trimmed.trim_start_matches("Name:").trim().to_string());
        } else if trimmed.starts_with("Description:") {
            cur_desc = Some(trimmed.trim_start_matches("Description:").trim().to_string());
        } else if trimmed.is_empty() && cur_name.is_some()
            && let Some(n) = cur_name.take() {
                sinks.push(Sink { name: n, description: cur_desc.take().unwrap_or_default() });
            }
    }
    if let Some(n) = cur_name.take() {
        sinks.push(Sink { name: n, description: cur_desc.take().unwrap_or_default() });
    }
    sinks
}

pub fn default_sink() -> Option<String> {
    let out = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C").args(["info"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("Default Sink:") {
            return Some(t.trim_start_matches("Default Sink:").trim().to_string());
        }
    }
    None
}

pub fn get_volume(sink: &str) -> Option<f32> {
    let out = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C")
        .args(["get-sink-volume", sink])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // "Volume: front-left: 30802 /  47% / -19.62 dB, ..."
    for part in text.split(',') {
        if let Some(idx) = part.find('%') {
            // find number before %
            let before = &part[..idx];
            let num_str = before.split_whitespace().last()?;
            if let Ok(v) = num_str.parse::<f32>() {
                return Some(v);
            }
        }
    }
    None
}

pub fn set_volume(sink: &str, percent: f32) {
    let p = percent.clamp(0.0, 200.0);
    let arg = format!("{}%", p as i32);
    let _ = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C")
        .args(["set-sink-volume", sink, &arg])
        .status();
}

pub fn get_mute(sink: &str) -> bool {
    let out = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C")
        .args(["get-sink-mute", sink])
        .output()
        .ok();
    let text = out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    text.to_lowercase().contains("yes")
}

pub fn set_mute(sink: &str, mute: bool) {
    let _ = Command::new("pactl").env("LC_ALL", "C").env("LANG", "C")
        .args(["set-sink-mute", sink, if mute { "1" } else { "0" }])
        .status();
}