//! SigmaOS-at — accessibility settings CLI.
//!
//! Usage:
//!   sigmaos-at status|profiles|config|env
//!   sigmaos-at set <key> <value>     dotted keys: speech.rate, display.zoom, ...
//!   sigmaos-at toggle <feature>      screen_reader|magnifier|high_contrast|reduced_motion|sticky_keys
//!   sigmaos-at profile <name>        apply preset (low-vision|screen-reader|motor)
//!   sigmaos-at list                  show all settings

use sigmaos_at::SigmaOSAccessibility;
use std::process::exit;

fn usage() -> ! {
    eprintln!("usage: sigmaos-at <status|list|profiles|config|env|set|toggle|profile> [args]");
    exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { usage(); }
    let state = SigmaOSAccessibility::state_path();
    let mut m = SigmaOSAccessibility::load_from(&state).unwrap_or_else(|_| SigmaOSAccessibility::new());
    let cmd = args[0].as_str();
    let rest = &args[1..];
    let arg = |i: usize| -> &str { rest.get(i).map(|s| s.as_str()).unwrap_or_else(|| usage()) };

    let result: Result<(), String> = (|| -> Result<(), String> { match cmd {
        "status" => { println!("{}", m.status_line()); Ok(()) }
        "list" => {
            let s = &m.settings;
            println!("screen_reader={} speech={}wpm pitch={}% vol={}% typing={}",
                s.screen_reader, s.speech_rate, s.speech_pitch, s.speech_volume, s.announce_typing);
            println!("magnifier={} zoom={}% hc={} cb={} rm={} lt={}",
                s.magnifier, s.zoom, s.high_contrast, s.colorblind.label(), s.reduced_motion, s.large_text);
            println!("sticky={} slow={} bounce={} mouse_keys={} kb_nav={}",
                s.sticky_keys, s.slow_keys, s.bounce_keys, s.mouse_keys, s.keyboard_nav);
            Ok(())
        }
        "profiles" => { for p in m.profiles() { println!("{}", p.name); } Ok(()) }
        "config" => { print!("{}", m.render_config()); Ok(()) }
        "env" => { print!("{}", m.render_env()); Ok(()) }
        "set" => { m.set(arg(0), arg(1)).map_err(|e| e.to_string())?; println!("{}={}", arg(0), arg(1)); Ok(()) }
        "toggle" => { let v = m.toggle(arg(0)).map_err(|e| e.to_string())?; println!("{}={}", arg(0), v); Ok(()) }
        "profile" => { m.apply_profile(arg(0)).map_err(|e| e.to_string())?; println!("applied {}", arg(0)); Ok(()) }
        _ => usage(),
    } })();

    match result {
        Ok(()) => { if let Err(e) = m.save_to(&state) { eprintln!("sigmaos-at: warn: save: {e}"); } }
        Err(e) => { eprintln!("sigmaos-at: {e}"); exit(1); }
    }
}
