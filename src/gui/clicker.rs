use enigo::Enigo;
use rand::thread_rng;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::modules::mouse::{get_next_click_area, generate_random_coordinates, simulate_human_movement, human_like_click, handle_sleep_period};
use crate::gui::app::{AppState, ClickerStatus};

pub struct ClickerThread {
    thread_handle: Option<JoinHandle<()>>,
    is_paused: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
}

impl ClickerThread {
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            is_paused: Arc::new(AtomicBool::new(false)),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, app_state: Arc<Mutex<AppState>>) {
        // Make sure we're not already running
        if self.thread_handle.is_some() {
            println!("Clicker thread is already running");
            return;
        }

        println!("Starting clicker thread");

        // Reset the flags
        self.is_paused.store(false, Ordering::SeqCst);
        self.should_stop.store(false, Ordering::SeqCst);

        // Clone the Arc pointers for the thread
        let is_paused = Arc::clone(&self.is_paused);
        let should_stop = Arc::clone(&self.should_stop);
        let app_state_clone = Arc::clone(&app_state);

        // Start the clicker thread
        self.thread_handle = Some(thread::spawn(move || {
            println!("Clicker thread started");
            let mut enigo = Enigo::new();
            let mut rng = thread_rng();
            let mut current_area_index = 0;

            // Main clicking loop
            while !should_stop.load(Ordering::SeqCst) {
                if !is_paused.load(Ordering::SeqCst) {
                    println!("Performing click operation");
                    // Get the current config
                    let config = match app_state_clone.lock() {
                        Ok(state) => state.current_config.clone(),
                        Err(e) => {
                            eprintln!("Failed to lock app state: {}", e);
                            continue;
                        }
                    };

                    // Get the next click area
                    let (area, (area_start_x, area_start_y)) = if config.multi_area.enabled {
                        println!("Using multi-area mode");
                        get_next_click_area(&config, &mut current_area_index, &mut rng)
                    } else {
                        println!("Using single area mode");
                        // Calculate the centered area if needed
                        let (start_x, start_y) = crate::modules::mouse::calculate_click_area(&enigo, &config.click_area);
                        (config.click_area.clone(), (start_x, start_y))
                    };

                    println!("Click area: {}x{} at ({}, {})", area.width, area.height, area_start_x, area_start_y);

                    // Generate random coordinates within the clicking area
                    let (x, y) = generate_random_coordinates(
                        area_start_x,
                        area_start_y,
                        area.width,
                        area.height,
                        &mut rng
                    );

                    println!("Clicking at position: ({}, {})", x, y);

                    // Simulate human-like mouse movement
                    if let Err(e) = simulate_human_movement(&mut enigo, x, y, &mut rng) {
                        eprintln!("Warning: Mouse movement failed: {}", e);
                        continue;
                    }

                    // Perform the click with human-like duration
                    if let Err(e) = human_like_click(&mut enigo, &mut rng, &config) {
                        eprintln!("Warning: Click action failed: {}", e);
                        continue;
                    }

                    // Update the click count
                    if let Ok(mut state) = app_state_clone.lock() {
                        state.click_count += 1;
                        println!("Click count: {}", state.click_count);
                    } else {
                        eprintln!("Failed to lock app state to update click count");
                    }

                    // Handle sleep period
                    println!("Sleeping before next click");
                    if let Err(e) = handle_sleep_period(&mut enigo, &mut rng, &is_paused, &should_stop, &config) {
                        eprintln!("Warning: Sleep period failed: {}", e);
                    }
                } else {
                    println!("Clicker is paused");
                    thread::sleep(Duration::from_millis(100));
                }
            }
            println!("Clicker thread stopped");
        }));

        // Update the app state
        let mut state = app_state.lock().unwrap();
        state.clicker_status = ClickerStatus::Running;
        state.start_time = Some(Instant::now());
        println!("Clicker status set to Running");
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn stop(&mut self) {
        if self.thread_handle.is_some() {
            // Signal the thread to stop
            self.should_stop.store(true, Ordering::SeqCst);

            // Wait for the thread to finish
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread_handle.is_some() && !self.should_stop.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }
}

impl Drop for ClickerThread {
    fn drop(&mut self) {
        self.stop();
    }
}
