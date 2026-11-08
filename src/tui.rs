use crate::chip::Chip;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io::{stdout, Stdout},
    time::{Duration, Instant},
};


pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_frame: Instant,
    last_timer: Instant,
    target_fps: u64,
    cycles_per_frame: u32,
}

impl Tui {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            last_frame: Instant::now(),
            last_timer: Instant::now(),
            target_fps: 60,
            cycles_per_frame: 10 
        })
    }
    pub fn run(&mut self, chip: &mut Chip) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if self.poll_input(chip)? {
                break;
            }

            for _ in 0..self.cycles_per_frame {
                chip.cycle();
            }

            if self.last_timer.elapsed() >= Duration::from_millis(16) {
                chip.tick_timer();
                self.last_timer = Instant::now();
            }

            if chip.draw_flag
                || self.last_frame.elapsed() >= Duration::from_millis(1000 / self.target_fps)
            {
                self.draw(chip)?;
                chip.draw_flag = false;
                self.last_frame = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn poll_input(&mut self, chip: &mut Chip) -> Result<bool, Box<dyn std::error::Error>> {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.kind {
                    KeyEventKind::Press => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                        _ => set_key(chip, key.code, true),
                    },
                    KeyEventKind::Release => set_key(chip, key.code, false),
                    _ => {}
                }
            }
        }
        Ok(false)
    }
    fn draw(&mut self, chip: &Chip) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let area = f.area();

            let mut lines = Vec::with_capacity(16);
            for y in (0..32).step_by(2) {
                let mut s = String::with_capacity(64);
                for x in 0..64 {
                    let top = chip.display[y][x];
                    let bottom = if y + 1 < 32 {
                        chip.display[y + 1][x]
                    } else {
                        false
                    };

                    let ch = match (top, bottom) {
                        (true, true) => '█',
                        (true, false) => '▀',
                        (false, true) => '▄',
                        (false, false) => ' ',
                    };
                    s.push(ch);
                }
                lines.push(Line::from(s));
            }

            let block = Block::default()
                    .title(" CHIP-8 TUI ")
                    .borders(Borders::ALL);
            let paragraph = Paragraph::new(lines).block(block);
            f.render_widget(paragraph, area);
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

fn set_key(chip: &mut Chip, code: KeyCode, pressed: bool) {
    let idx = match code {
        KeyCode::Char('1') => 0x1,
        KeyCode::Char('2') => 0x2,
        KeyCode::Char('3') => 0x3,
        KeyCode::Char('4') => 0xC,
        KeyCode::Char('q') => 0x4,
        KeyCode::Char('w') => 0x5,
        KeyCode::Char('e') => 0x6,
        KeyCode::Char('r') => 0xD,
        KeyCode::Char('a') => 0x7,
        KeyCode::Char('s') => 0x8,
        KeyCode::Char('d') => 0x9,
        KeyCode::Char('f') => 0xE,
        KeyCode::Char('z') => 0xA,
        KeyCode::Char('x') => 0x0,
        KeyCode::Char('c') => 0xB,
        KeyCode::Char('v') => 0xF,
        _ => return,
    };
    chip.keys[idx] = pressed;
}


