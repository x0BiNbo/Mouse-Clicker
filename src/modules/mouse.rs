use enigo::{Enigo, MouseControllable};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::{
    Win32::UI::WindowsAndMessaging::{
        SetCursorPos, GetForegroundWindow, SetForegroundWindow,
        GetWindowRect,
    },
    Win32::Foundation::RECT,
    Win32::System::Console::GetConsoleWindow,
};
use crate::modules::error::Result;
use crate::modules::ui::encode_text;
use crate::modules::config::{Config, ClickArea, AreaSelectionMode};

pub fn calculate_centered_area(enigo: &Enigo, width: i32, height: i32) -> (i32, i32) {
    let screen_size = enigo.main_display_size();
    let screen_width = screen_size.0 as i32;
    let screen_height = screen_size.1 as i32;
    let start_x = (screen_width - width) / 2;
    let start_y = (screen_height - height) / 2;
    (start_x, start_y)
}

pub fn calculate_click_area(enigo: &Enigo, area: &ClickArea) -> (i32, i32) {
    if area.centered {
        calculate_centered_area(enigo, area.width, area.height)
    } else {
        (area.x_offset, area.y_offset)
    }
}

pub fn get_next_click_area(
    config: &Config,
    current_index: &mut usize,
    rng: &mut impl Rng
) -> (ClickArea, (i32, i32)) {
    // If multi-area is not enabled, use the primary click area
    if !config.multi_area.enabled || config.multi_area.areas.is_empty() {
        return (config.click_area.clone(), calculate_click_area(&Enigo::new(), &config.click_area));
    }

    // Get the next area based on the selection mode
    let (area, _) = match config.multi_area.selection_mode {
        AreaSelectionMode::Sequential => {
            // Get the next area in sequence
            let area = &config.multi_area.areas[*current_index].0;

            // Update the index for next time
            *current_index = (*current_index + 1) % config.multi_area.areas.len();

            (area.clone(), 1.0)
        },
        AreaSelectionMode::Random => {
            // Pick a random area
            let index = rng.gen_range(0..config.multi_area.areas.len());
            let area = &config.multi_area.areas[index].0;

            (area.clone(), 1.0)
        },
        AreaSelectionMode::Weighted => {
            // Calculate total weight
            let total_weight: f32 = config.multi_area.areas.iter()
                .map(|(_, weight)| *weight)
                .sum();

            // Generate a random value between 0 and total_weight
            let mut random_value = rng.gen_range(0.0..total_weight);

            // Find the area based on the random value
            let mut selected_index = 0;

            for (i, (_, weight)) in config.multi_area.areas.iter().enumerate() {
                if random_value <= *weight {
                    selected_index = i;
                    break;
                }
                random_value -= weight;
            }

            let (area, weight) = &config.multi_area.areas[selected_index];
            (area.clone(), *weight)
        },
    };

    // Calculate the coordinates for the selected area
    let coords = calculate_click_area(&Enigo::new(), &area);

    (area, coords)
}

pub fn generate_random_coordinates(
    start_x: i32,
    start_y: i32,
    width: i32,
    height: i32,
    rng: &mut impl Rng,
) -> (i32, i32) {
    let x = rng.gen_range(start_x..start_x + width);
    let y = rng.gen_range(start_y..start_y + height);
    (x, y)
}

pub fn safe_move_mouse(x: i32, y: i32) -> bool {
    unsafe {
        // Get the current foreground window
        let foreground = GetForegroundWindow();

        // Try to get the window dimensions
        let mut rect = RECT::default();
        if GetWindowRect(foreground, &mut rect).is_ok() {
            // Check if our coordinates are within the window
            if x < rect.left || x > rect.right || y < rect.top || y > rect.bottom {
                // If we're moving outside the current window, try to set foreground
                let hwnd = GetConsoleWindow();
                if hwnd.0 != 0 {
                    SetForegroundWindow(hwnd);
                }
            }
        }

        SetCursorPos(x, y).is_ok()
    }
}

pub fn simulate_human_movement(
   enigo: &mut Enigo,
   target_x: i32,
   target_y: i32,
   _rng: &mut impl Rng,
) -> Result<()> {
   let screen_size = enigo.main_display_size();
   let max_x = screen_size.0 as i32;
   let max_y = screen_size.1 as i32;

   // Clamp target coordinates
   let target_x = target_x.clamp(0, max_x - 1);
   let target_y = target_y.clamp(0, max_y - 1);

   let start_pos = enigo.mouse_location();
   let dx = target_x - start_pos.0;
   let dy = target_y - start_pos.1;

   // If movement is too small, skip it
   if dx.abs() < 2 && dy.abs() < 2 {
       return Ok(());
   }

   let distance = ((dx * dx + dy * dy) as f64).sqrt();
   let steps = (distance / 10.0).ceil() as usize;
   let step_time = Duration::from_millis(2);

   for i in 1..=steps {
       let progress = i as f64 / steps as f64;

       // Simple linear interpolation
       let x = (start_pos.0 as f64 + dx as f64 * progress) as i32;
       let y = (start_pos.1 as f64 + dy as f64 * progress) as i32;

       // Clamp coordinates
       let x = x.clamp(0, max_x - 1);
       let y = y.clamp(0, max_y - 1);

       // Use Windows API directly
       if !safe_move_mouse(x, y) {
            let error_msg = format!("Failed to move mouse to ({}, {})", x, y);
            eprintln!("{} | {}", encode_text(&error_msg), error_msg);
            continue;
       }

       thread::sleep(step_time);
   }

   Ok(())
}

pub fn simulate_idle_movement(enigo: &mut Enigo, rng: &mut impl Rng) -> Result<()> {
    // Reduce the frequency of idle movements significantly
    if rng.gen_bool(0.001) {
        let screen_size = enigo.main_display_size();
        let current_pos = enigo.mouse_location();

        // Ensure new position is within screen bounds
        let new_x = (current_pos.0 + rng.gen_range(-1..=1))
            .clamp(0, screen_size.0 as i32 - 1);
        let new_y = (current_pos.1 + rng.gen_range(-1..=1))
            .clamp(0, screen_size.1 as i32 - 1);

        // Only move if the position has actually changed
        if new_x != current_pos.0 || new_y != current_pos.1 {
            if !safe_move_mouse(new_x, new_y) {
                eprintln!("Failed to perform idle movement");
            }
        }
    }
    Ok(())
}

pub fn get_click_type(rng: &mut impl Rng, config: &Config) -> crate::modules::config::ClickType {
    use crate::modules::config::ClickType;

    if !config.click_options.randomize_click_type {
        return config.click_options.click_type;
    }

    // Calculate total weight
    let total_weight: f32 = config.click_options.click_type_weights.iter()
        .map(|(_, weight)| weight)
        .sum();

    // Generate a random value between 0 and total_weight
    let mut random_value = rng.gen_range(0.0..total_weight);

    // Find the click type based on the random value
    for (click_type, weight) in &config.click_options.click_type_weights {
        if random_value <= *weight {
            return *click_type;
        }
        random_value -= weight;
    }

    // Fallback to single click
    ClickType::Single
}

pub fn human_like_click(enigo: &mut Enigo, rng: &mut impl Rng, config: &Config) -> Result<()> {
    let normal = Normal::new(
        config.click_timing.click_duration_mean,
        config.click_timing.click_duration_std_dev
    ).unwrap();

    let click_duration = normal.sample(rng) as f64;
    let clamped_duration = click_duration.clamp(40.0, 150.0) as u64;

    let click_type = get_click_type(rng, config);

    match click_type {
        crate::modules::config::ClickType::Single => {
            enigo.mouse_down(enigo::MouseButton::Left);
            thread::sleep(Duration::from_millis(clamped_duration));
            enigo.mouse_up(enigo::MouseButton::Left);
        },
        crate::modules::config::ClickType::Double => {
            // First click
            enigo.mouse_down(enigo::MouseButton::Left);
            thread::sleep(Duration::from_millis(clamped_duration));
            enigo.mouse_up(enigo::MouseButton::Left);

            // Gap between clicks
            thread::sleep(Duration::from_millis(config.click_timing.double_click_gap));

            // Second click
            enigo.mouse_down(enigo::MouseButton::Left);
            thread::sleep(Duration::from_millis(clamped_duration));
            enigo.mouse_up(enigo::MouseButton::Left);
        },
        crate::modules::config::ClickType::Right => {
            enigo.mouse_down(enigo::MouseButton::Right);
            thread::sleep(Duration::from_millis(clamped_duration));
            enigo.mouse_up(enigo::MouseButton::Right);
        },
        crate::modules::config::ClickType::Middle => {
            enigo.mouse_down(enigo::MouseButton::Middle);
            thread::sleep(Duration::from_millis(clamped_duration));
            enigo.mouse_up(enigo::MouseButton::Middle);
        },
    }

    Ok(())
}

pub fn handle_sleep_period(
    enigo: &mut Enigo,
    rng: &mut impl Rng,
    is_paused: &AtomicBool,
    should_quit: &AtomicBool,
    config: &Config,
) -> Result<()> {
    let sleep_duration = Duration::from_secs_f32(
        rng.gen_range(config.click_timing.min_delay..config.click_timing.max_delay)
    );
    let sleep_start = Instant::now();

    while sleep_start.elapsed() < sleep_duration
        && !is_paused.load(Ordering::SeqCst)
        && !should_quit.load(Ordering::SeqCst)
    {
        thread::sleep(Duration::from_millis(100));

        // Ignore any errors from idle movement
        let _ = simulate_idle_movement(enigo, rng);
    }
    Ok(())
}
