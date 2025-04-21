use std::time::Instant;
use crate::modules::error::Result;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    execute,
};
use std::io;

pub fn print_banner() {
    println!(r#"
|  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  |
|   ███╗   ███╗ ██████╗ ██╗   ██╗███████╗███████╗  ███████╗██╗     ██╗ ██████╗██╗  ██╗ |
|   ████╗ ████║██╔═══██╗██║   ██║██╔════╝██╔════╝  ██╔════╝██║     ██║██╔════╝██║ ██╔╝ |
|   ██╔████╔██║██║   ██║██║   ██║███████╗█████╗    █████╗  ██║     ██║██║     █████╔╝  |
|   ██║╚██╔╝██║██║   ██║██║   ██║╚════██║██╔══╝    ██╔══╝  ██║     ██║██║     ██╔═██╗  |
|   ██║ ╚═╝ ██║╚██████╔╝╚██████╔╝███████║███████╗  ██║     ███████╗██║╚██████╗██║  ██╗ |
|   ╚═╝     ╚═╝ ╚═════╝  ╚═════╝ ╚══════╝╚══════╝  ╚═╝     ╚══════╝╚═╝ ╚═════╝╚═╝  ╚═╝ |
|                                                                                      |
|   -- --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  --  |
    "#);
}

pub fn print_summary(start_time: Instant, click_count: u32) {
    let duration = start_time.elapsed();

    let msg1 = format!("Program finished. Total clicks: {}", click_count);
    println!("\n{} | {}", encode_text(&msg1), msg1);

    let msg2 = format!("Total runtime: {:.2} seconds", duration.as_secs_f32());
    println!("{} | {}", encode_text(&msg2), msg2);

    let msg3 = format!(
        "Average clicks per minute: {:.2}",
        click_count as f32 / (duration.as_secs() as f32 / 60.0)
    );
    println!("{} | {}", encode_text(&msg3), msg3);
}

pub fn clear_screen() -> Result<()> {
    let (_, rows) = crossterm::terminal::size()
        .map_err(|e| crate::modules::error::AppError::IoError(e))?;

    execute!(
        io::stdout(),
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown),
        cursor::MoveTo(0, rows - 1),
    ).map_err(|e| crate::modules::error::AppError::IoError(e))?;

    Ok(())
}

pub fn encode_text(text: &str) -> String {
    let digest = md5::compute(text.as_bytes());
    format!("{:x}", digest)
}
