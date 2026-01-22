mod tui;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if we have a TTY before trying to run TUI
    if !atty::is(atty::Stream::Stdout) {
        println!("🚨 SDR CONTROL TERMINAL 🚨");
        println!("═══════════════════════════");
        println!();
        println!("❌ ERROR: No TTY detected!");
        println!("This is a Terminal User Interface (TUI) application that requires a proper terminal.");
        println!();
        println!("Please run this in:");
        println!("• A real terminal emulator (Terminal, iTerm, etc.)");
        println!("• Not in an IDE output panel");
        println!("• Not in a web-based environment");
        println!();
        println!("For demo purposes, here's what the interface would show:");
        println!();
        println!("🛰️  SDR CONTROL TERMINAL  🛰️");
        println!("┌─────────────────────────────────────┐");
        println!("│ 📡 FREQUENCY ⚡ GAIN 📊 SAMPLE RATE │");
        println!("├─────────────────────────────────────┤");
        println!("│ Frequency: 100.000 MHz              │");
        println!("│ Use ↑↓ to adjust                    │");
        println!("│ Step: 1 MHz                         │");
        println!("├─────────────────────────────────────┤");
        println!("│ [C] Connect USRP  [S] Start Streaming │");
        println!("│ [Q] Quit                             │");
        println!("└─────────────────────────────────────┘");
        println!();
        println!("STATUS: DEMO MODE - Mock SDR data active");
        println!("STREAMING: INACTIVE");
        return Ok(());
    }

    // Launch the futuristic SDR TUI
    tui::run_tui()?;
    Ok(())
}