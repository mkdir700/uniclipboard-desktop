use anyhow::{anyhow, Result};
use clipboard_rs::{ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};
use std::env;
use std::io::Write;
use std::sync::mpsc::{self, Sender};
use std::time::Instant;
use uc_core::ports::LocalClipboardPort;
use uc_core::{SystemClipboardRepresentation, SystemClipboardSnapshot};
use uc_platform::clipboard::LocalClipboard;

struct ProbeEvent {
    observed_ms: i64,
    observed_instant: Instant,
    snapshot: Option<SystemClipboardSnapshot>,
    error: Option<String>,
}

struct ProbeHandler {
    clipboard: LocalClipboard,
    tx: Sender<ProbeEvent>,
}

impl ClipboardHandler for ProbeHandler {
    fn on_clipboard_change(&mut self) {
        let observed_ms = chrono::Utc::now().timestamp_millis();
        let observed_instant = Instant::now();
        let event = match self.clipboard.read_snapshot() {
            Ok(snapshot) => ProbeEvent {
                observed_ms,
                observed_instant,
                snapshot: Some(snapshot),
                error: None,
            },
            Err(err) => ProbeEvent {
                observed_ms,
                observed_instant,
                snapshot: None,
                error: Some(err.to_string()),
            },
        };

        let _ = self.tx.send(event);
    }
}

fn main() -> Result<()> {
    let _ = log_line("main: entry");
    let _ = log_line(&format!("main: args={:?}", env::args().collect::<Vec<_>>()));

    let max_events = parse_max_events()?;

    println!("clipboard_probe: starting");
    println!(
        "- max_events: {}",
        max_events.map_or("none".into(), |v| v.to_string())
    );
    println!("- stop: Ctrl+C");

    let (tx, rx) = mpsc::channel();

    let clipboard = LocalClipboard::new()?;
    match clipboard.read_snapshot() {
        Ok(snapshot) => {
            println!("\ninitial snapshot");
            print_snapshot(&snapshot);
        }
        Err(err) => {
            println!("\ninitial snapshot error: {err}");
        }
    }

    let handler = ProbeHandler { clipboard, tx };
    let mut watcher = ClipboardWatcherContext::new().map_err(|e| anyhow!(e))?;
    watcher.add_handler(handler);

    std::thread::spawn(move || {
        println!("\nclipboard watcher: started");
        watcher.start_watch();
        println!("clipboard watcher: stopped");
    });

    let mut last_event_instant: Option<Instant> = None;
    let mut last_fingerprint: Option<u64> = None;
    let mut same_streak: usize = 0;
    let mut event_count: usize = 0;

    while let Ok(event) = rx.recv() {
        event_count += 1;

        let delta_ms = last_event_instant
            .map(|instant| event.observed_instant.duration_since(instant).as_millis());
        last_event_instant = Some(event.observed_instant);

        println!("\nevent #{event_count}");
        println!("- observed: {}", format_ms(event.observed_ms));
        println!(
            "- delta_ms: {}",
            delta_ms.map_or("n/a".into(), |v| v.to_string())
        );

        if let Some(err) = event.error {
            println!("- read_snapshot error: {err}");
        } else if let Some(snapshot) = event.snapshot {
            let fingerprint = snapshot_fingerprint(&snapshot);
            if last_fingerprint == Some(fingerprint) {
                same_streak += 1;
            } else {
                same_streak = 0;
            }
            last_fingerprint = Some(fingerprint);

            println!("- same_content_streak: {same_streak}");
            print_snapshot(&snapshot);
        }

        if let Some(limit) = max_events {
            if event_count >= limit {
                println!("\nmax_events reached, exiting");
                break;
            }
        }
    }

    Ok(())
}

fn log_line(line: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("clipboard_probe.log")?;
    writeln!(file, "{} {}", chrono::Utc::now().to_rfc3339(), line)?;
    Ok(())
}

fn parse_max_events() -> Result<Option<usize>> {
    let mut max_events: Option<usize> = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-events" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--max-events expects a value"))?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| anyhow!("--max-events expects a number, got: {value}"))?;
                max_events = Some(parsed);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                return Err(anyhow!("unknown argument: {arg} (use --help for usage)"));
            }
        }
    }

    Ok(max_events)
}

fn print_usage() {
    println!("clipboard_probe usage:");
    println!("  --max-events <n>   stop after n events");
    println!("  --help, -h         show this help");
}

fn print_snapshot(snapshot: &SystemClipboardSnapshot) {
    println!(
        "- snapshot.ts_ms: {} ({})",
        snapshot.ts_ms,
        format_ms(snapshot.ts_ms)
    );
    println!("- representations: {}", snapshot.representations.len());

    for (idx, rep) in snapshot.representations.iter().enumerate() {
        let desc = describe_representation(rep);
        println!("  rep[{idx}]: {desc}");
    }
}

fn describe_representation(rep: &SystemClipboardRepresentation) -> String {
    let mime = rep.mime.as_ref().map(|m| m.as_str()).unwrap_or("-");
    let preview = if is_text_representation(mime, &rep.format_id) {
        format!("\"{}\"", text_preview(&rep.bytes, 160))
    } else {
        format!("hex:{}", hex_preview(&rep.bytes, 24))
    };

    format!(
        "format_id={} mime={} bytes={} preview={}",
        rep.format_id,
        mime,
        rep.bytes.len(),
        preview
    )
}

fn is_text_representation(mime: &str, format_id: &str) -> bool {
    if mime.starts_with("text/") {
        return true;
    }

    matches!(format_id, "text" | "rtf" | "html" | "files")
}

fn text_preview(bytes: &[u8], max_len: usize) -> String {
    let clipped_len = bytes.len().min(max_len);
    let text = String::from_utf8_lossy(&bytes[..clipped_len]);
    let mut escaped = text.escape_default().to_string();

    if bytes.len() > max_len {
        escaped.push_str("...");
    }

    escaped
}

fn hex_preview(bytes: &[u8], max_len: usize) -> String {
    if bytes.is_empty() {
        return "(empty)".to_string();
    }

    let mut out = String::new();
    for (idx, byte) in bytes.iter().take(max_len).enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{:02x}", byte));
    }

    if bytes.len() > max_len {
        out.push_str(" ...");
    }

    out
}

fn snapshot_fingerprint(snapshot: &SystemClipboardSnapshot) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    snapshot.representations.len().hash(&mut hasher);

    for rep in &snapshot.representations {
        rep.format_id.hash(&mut hasher);
        rep.mime.hash(&mut hasher);
        rep.bytes.hash(&mut hasher);
    }

    hasher.finish()
}

fn format_ms(ms: i64) -> String {
    use chrono::TimeZone;

    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| ms.to_string())
}
