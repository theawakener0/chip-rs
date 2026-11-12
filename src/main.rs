mod chip;
mod tui;

use chip::Chip;
use tui::Tui;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom>", args[0]);
        std::process::exit(1);
    }

    let rom = fs::read(&args[1])?;

    let mut chip = Chip::new();
    chip.load_rom(&rom);

    let mut tui = Tui::new()?;
    tui.run(&mut chip, &rom)?;

    Ok(())
}
