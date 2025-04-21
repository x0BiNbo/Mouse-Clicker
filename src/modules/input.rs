use std::io::{self, Write};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use regex::Regex;
use crate::modules::error::{AppError, Result};
use crate::modules::ui::{encode_text, clear_screen};

pub fn get_validated_input(min: i32, max: i32) -> Result<i32> {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().parse::<i32>() {
            Ok(n) if n >= min && n <= max => return Ok(n),
            Ok(_) => println!("Please enter a number between {} and {}", min, max),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

pub fn spawn_keyboard_handler(
    is_paused: Arc<AtomicBool>,
    should_quit: Arc<AtomicBool>,
) -> Result<JoinHandle<Result<()>>> {
    Ok(thread::spawn(move || -> Result<()> {
        while !should_quit.load(Ordering::SeqCst) {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char('p') => {
                                let was_paused = is_paused.fetch_xor(true, Ordering::SeqCst);
                                let msg = if was_paused { "Resumed" } else { "Paused" };
                                println!("{} | {}", encode_text(msg), msg);
                            }
                            KeyCode::Char('c') => {
                                clear_screen()?;
                            }
                            KeyCode::Char('q') => {
                                should_quit.store(true, Ordering::SeqCst);
                                break;
                            }
                            KeyCode::Char('t') => {
                                handle_timed_pause(&is_paused)?
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }))
}

pub fn handle_timed_pause(is_paused: &AtomicBool) -> Result<()> {
    println!("Enter the pause duration (e.g., 1m30s): ");
    io::stdout().flush()?;
    disable_raw_mode()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    enable_raw_mode()?;

    let duration = parse_duration(&input.trim()).unwrap_or_else(|_| {
        println!("Invalid duration format. Using default duration of 1 minute.");
        Duration::from_secs(60)
    });
    
    is_paused.store(true, Ordering::SeqCst);
    println!("Paused for {:?}", duration);
    thread::sleep(duration);
    is_paused.store(false, Ordering::SeqCst);
    println!("Resumed");
    Ok(())
}

pub fn parse_duration(s: &str) -> Result<Duration> {
    let re = Regex::new(r"(?i)^((?P<m>\d+)m)?((?P<s>\d+)s)?$")
        .map_err(|e| AppError::ParseError(e.to_string()))?;
        
    if let Some(caps) = re.captures(s) {
        let minutes = caps
            .name("m")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let seconds = caps
            .name("s")
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        
        if minutes == 0 && seconds == 0 {
            return Err(AppError::ParseError("Duration must be greater than 0".into()));
        }
        
        Ok(Duration::from_secs(minutes * 60 + seconds))
    } else {
        Err(AppError::ParseError("Invalid duration format".into()))
    }
}
