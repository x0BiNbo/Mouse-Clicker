use egui::{Ui, Color32, Stroke, Vec2, Pos2, RichText};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::gui::app::AppState;
use crate::gui::theme::AppTheme;
use crate::gui::components;

pub struct StatsView {
    state: Arc<Mutex<AppState>>,
    click_history: Vec<(f32, f32)>, // (time, clicks per minute)
}

impl StatsView {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            click_history: Vec::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        let (click_count, start_time, elapsed_seconds, clicks_per_minute) = {
            let state = self.state.lock().unwrap();
            let click_count = state.click_count;
            let start_time = state.start_time;

            let (elapsed_seconds, clicks_per_minute) = if let Some(start) = start_time {
                let elapsed = start.elapsed();
                let elapsed_seconds = elapsed.as_secs_f32();
                let clicks_per_minute = if elapsed_seconds > 0.0 {
                    (click_count as f32 / elapsed_seconds) * 60.0
                } else {
                    0.0
                };
                (elapsed_seconds, clicks_per_minute)
            } else {
                (0.0, 0.0)
            };

            (click_count, start_time, elapsed_seconds, clicks_per_minute)
        };

        // Update click history
        if let Some(_) = start_time {
            // Add a data point every few seconds
            if self.click_history.is_empty() ||
               (self.click_history.last().unwrap().0 + 5.0) < elapsed_seconds {
                self.click_history.push((elapsed_seconds, clicks_per_minute));
            }
        }

        // Current Session Stats Card
        components::card(ui, &theme, "Current Session Statistics", |ui| {
            ui.add_space(8.0);

            // Create a grid layout for stats
            egui::Grid::new("stats_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    // Total Clicks
                    ui.label(RichText::new("Total Clicks:").strong());
                    ui.label(RichText::new(format!("{}", click_count)).size(18.0));
                    ui.end_row();

                    // Running Time
                    ui.label(RichText::new("Running Time:").strong());
                    if let Some(_) = start_time {
                        ui.label(RichText::new(format!("{:.1} seconds", elapsed_seconds)).size(18.0));
                    } else {
                        ui.label(RichText::new("Not running").italics());
                    }
                    ui.end_row();

                    // Clicks per Minute
                    ui.label(RichText::new("Clicks per Minute:").strong());
                    if let Some(_) = start_time {
                        ui.label(RichText::new(format!("{:.2}", clicks_per_minute)).size(18.0));
                    } else {
                        ui.label(RichText::new("N/A").italics());
                    }
                    ui.end_row();

                    // Efficiency (just a fun metric)
                    ui.label(RichText::new("Efficiency:").strong());
                    if start_time.is_some() && clicks_per_minute > 0.0 {
                        let efficiency = (clicks_per_minute / 300.0).min(1.0) * 100.0; // Assuming 300 CPM is max
                        let efficiency_text = format!("{:.1}%", efficiency);
                        let color = if efficiency > 80.0 {
                            theme.success
                        } else if efficiency > 40.0 {
                            theme.warning
                        } else {
                            theme.accent
                        };
                        ui.label(RichText::new(efficiency_text).color(color).size(18.0));
                    } else {
                        ui.label(RichText::new("N/A").italics());
                    }
                    ui.end_row();
                });

            ui.add_space(16.0);

            // Reset button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if components::accent_button(ui, &theme, "Reset Statistics") {
                    let mut state = self.state.lock().unwrap();
                    state.click_count = 0;
                    state.start_time = Some(Instant::now());
                    self.click_history.clear();
                }
            });
        });

        ui.add_space(16.0);

        // Click Rate Graph Card
        components::card(ui, &theme, "Click Rate Over Time", |ui| {
            // Draw the graph
            let graph_height = 220.0;
            let graph_width = ui.available_width();

            let (response, painter) = ui.allocate_painter(Vec2::new(graph_width, graph_height), egui::Sense::hover());

            // Draw graph background
            painter.rect_filled(
                response.rect,
                8.0,
                theme.card_background,
            );

            // Draw grid lines
            let grid_stroke = Stroke::new(0.5, theme.border);
            let num_grid_lines = 5;

            // Horizontal grid lines
            for i in 0..=num_grid_lines {
                let y = response.rect.top() + (i as f32 / num_grid_lines as f32) * response.rect.height();
                painter.line_segment(
                    [Pos2::new(response.rect.left(), y), Pos2::new(response.rect.right(), y)],
                    grid_stroke,
                );
            }

            // Vertical grid lines
            for i in 0..=num_grid_lines {
                let x = response.rect.left() + (i as f32 / num_grid_lines as f32) * response.rect.width();
                painter.line_segment(
                    [Pos2::new(x, response.rect.top()), Pos2::new(x, response.rect.bottom())],
                    grid_stroke,
                );
            }

            // Draw axes
            let axis_stroke = Stroke::new(1.5, theme.border);

            // X-axis
            painter.line_segment(
                [
                    Pos2::new(response.rect.left(), response.rect.bottom()),
                    Pos2::new(response.rect.right(), response.rect.bottom()),
                ],
                axis_stroke,
            );

            // Y-axis
            painter.line_segment(
                [
                    Pos2::new(response.rect.left(), response.rect.top()),
                    Pos2::new(response.rect.left(), response.rect.bottom()),
                ],
                axis_stroke,
            );

            // Draw data points if we have any
            if !self.click_history.is_empty() {
                // Find max values for scaling
                let max_time = self.click_history.last().unwrap().0.max(60.0); // At least 60 seconds
                let max_cpm = self.click_history.iter().map(|(_, cpm)| *cpm).fold(0.0, f32::max).max(10.0); // At least 10 CPM

                // Draw the line graph
                let points: Vec<Pos2> = self.click_history.iter().map(|(time, cpm)| {
                    let x = response.rect.left() + (time / max_time) * graph_width;
                    let y = response.rect.bottom() - (cpm / max_cpm) * graph_height;
                    Pos2::new(x, y)
                }).collect();

                if points.len() > 1 {
                    // Draw area under the curve
                    let mut fill_points = points.clone();
                    fill_points.push(Pos2::new(points.last().unwrap().x, response.rect.bottom()));
                    fill_points.push(Pos2::new(points.first().unwrap().x, response.rect.bottom()));

                    painter.add(egui::Shape::convex_polygon(
                        fill_points,
                        theme.primary.linear_multiply(0.2),
                        Stroke::NONE,
                    ));

                    // Draw the line
                    for i in 0..points.len() - 1 {
                        painter.line_segment(
                            [points[i], points[i + 1]],
                            Stroke::new(2.5, theme.primary)
                        );
                    }

                    // Draw points
                    for point in &points {
                        painter.circle_filled(*point, 4.0, theme.secondary);
                    }
                }

                // Draw axis labels with better formatting
                for i in 0..=num_grid_lines {
                    // Y-axis labels (CPM)
                    let cpm_value = max_cpm * (1.0 - i as f32 / num_grid_lines as f32);
                    let y_pos = response.rect.top() + (i as f32 / num_grid_lines as f32) * response.rect.height();

                    painter.text(
                        Pos2::new(response.rect.left() - 5.0, y_pos),
                        egui::Align2::RIGHT_CENTER,
                        format!("{:.0}", cpm_value),
                        egui::FontId::proportional(12.0),
                        theme.text,
                    );

                    // X-axis labels (Time)
                    let time_value = max_time * (i as f32 / num_grid_lines as f32);
                    let x_pos = response.rect.left() + (i as f32 / num_grid_lines as f32) * response.rect.width();

                    painter.text(
                        Pos2::new(x_pos, response.rect.bottom() + 12.0),
                        egui::Align2::CENTER_TOP,
                        format!("{:.0}s", time_value),
                        egui::FontId::proportional(12.0),
                        theme.text,
                    );
                }

                // Add axis titles
                painter.text(
                    Pos2::new(response.rect.left() - 25.0, response.rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    "CPM",
                    egui::FontId::proportional(14.0),
                    theme.text,
                );

                painter.text(
                    Pos2::new(response.rect.center().x, response.rect.bottom() + 25.0),
                    egui::Align2::CENTER_CENTER,
                    "Time (seconds)",
                    egui::FontId::proportional(14.0),
                    theme.text,
                );
            } else {
                // No data message with better styling
                painter.text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No data available - start clicking to see statistics",
                    egui::FontId::proportional(16.0),
                    theme.muted_text,
                );
            }
        });

        ui.add_space(16.0);

        // Performance Insights Card
        components::card(ui, &theme, "Performance Insights", |ui| {
            if !self.click_history.is_empty() && clicks_per_minute > 0.0 {
                // Calculate some insights
                let avg_cpm = self.click_history.iter().map(|(_, cpm)| *cpm).sum::<f32>() / self.click_history.len() as f32;
                let max_cpm = self.click_history.iter().map(|(_, cpm)| *cpm).fold(0.0, f32::max);
                let consistency = 1.0 - ((max_cpm - avg_cpm) / max_cpm).min(1.0);

                ui.add_space(8.0);

                // Progress bars for different metrics
                ui.label(RichText::new("Average Clicks per Minute").strong());
                components::progress_bar(ui, &theme, avg_cpm / 300.0, Some(&format!("{:.1} CPM", avg_cpm)));
                ui.add_space(8.0);

                ui.label(RichText::new("Peak Performance").strong());
                components::progress_bar(ui, &theme, max_cpm / 300.0, Some(&format!("{:.1} CPM", max_cpm)));
                ui.add_space(8.0);

                ui.label(RichText::new("Consistency").strong());
                components::progress_bar(ui, &theme, consistency, Some(&format!("{:.1}%", consistency * 100.0)));
                ui.add_space(8.0);

                // Performance tips
                ui.add_space(8.0);
                ui.label(RichText::new("Tips for Improvement").strong().size(16.0));
                ui.add_space(4.0);

                let tips = if avg_cpm < 60.0 {
                    vec![
                        "Try using keyboard shortcuts instead of clicking",
                        "Consider adjusting your mouse sensitivity",
                        "Practice with timing exercises to improve speed"
                    ]
                } else if consistency < 0.7 {
                    vec![
                        "Focus on maintaining a steady rhythm",
                        "Take short breaks to prevent fatigue",
                        "Try to maintain consistent pressure on mouse buttons"
                    ]
                } else {
                    vec![
                        "Great job! Your clicking performance is excellent",
                        "Try challenging yourself with more complex click patterns",
                        "Share your techniques with others to help them improve"
                    ]
                };

                for tip in tips {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("•").color(theme.secondary));
                        ui.label(tip);
                    });
                }
            } else {
                ui.add_space(10.0);
                ui.label("Start a clicking session to see performance insights.");
                ui.add_space(10.0);
            }
        });
    }
}
