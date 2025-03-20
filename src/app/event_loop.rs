use std::time::Instant;

use crate::utils::ease_out_cubic;

use super::MainApp;

pub(super) enum ContentState {
    Idle,
    Entering { start_time: Instant, progress: f32 },
    Showing { start_time: Instant },
    Exiting { start_time: Instant, progress: f32 },
}

pub(super) enum WindowState {
    Hidden,
    Entering { start_time: Instant, progress: f32 },
    Showing { start_time: Instant },
    Exiting { start_time: Instant, progress: f32 },
}

impl MainApp {
    pub(super) fn clear_content(&mut self) {
        self.icon = None;
        self.title = None;
        self.slider = None;
        self.description = None;
    }

    pub(super) fn reset(&mut self) {
        self.clear_content();
        self.content_state = ContentState::Idle;
        self.window_state = WindowState::Hidden;
        self.show_duration = self.config.globals.show_duration.unwrap_or(5.0);
    }

    pub(super) fn update_animation_states(&mut self, current_time: Instant) {
        let animation_duration = self.config.globals.animation_duration.unwrap_or(1.0);
        let show_duration = self.show_duration;

        // Actualizar estado de la ventana
        self.window_state = match &self.window_state {
            WindowState::Hidden => WindowState::Hidden,

            WindowState::Entering { start_time, .. } => {
                let elapsed = current_time.duration_since(*start_time).as_secs_f32();
                if elapsed >= animation_duration {
                    WindowState::Showing {
                        start_time: current_time,
                    }
                } else {
                    WindowState::Entering {
                        start_time: *start_time,
                        progress: elapsed / animation_duration,
                    }
                }
            }

            WindowState::Showing { start_time } => {
                let elapsed = current_time.duration_since(*start_time).as_secs_f32();
                if elapsed >= show_duration {
                    // Sincronizar la salida del contenido con la ventana
                    self.content_state = ContentState::Exiting {
                        start_time: current_time,
                        progress: 0.0,
                    };
                    WindowState::Exiting {
                        start_time: current_time,
                        progress: 0.0,
                    }
                } else {
                    WindowState::Showing {
                        start_time: *start_time,
                    }
                }
            }

            WindowState::Exiting { start_time, .. } => {
                let elapsed = current_time.duration_since(*start_time).as_secs_f32();
                if elapsed >= animation_duration {
                    WindowState::Hidden
                } else {
                    WindowState::Exiting {
                        start_time: *start_time,
                        progress: elapsed / animation_duration,
                    }
                }
            }
        };

        // La actualización del estado del contenido ahora sigue al estado de la ventana
        self.content_state = match &self.content_state {
            ContentState::Idle => ContentState::Idle,

            ContentState::Entering { start_time, .. } => {
                let elapsed = current_time.duration_since(*start_time).as_secs_f32();
                if elapsed >= animation_duration {
                    ContentState::Showing {
                        start_time: current_time,
                    }
                } else {
                    ContentState::Entering {
                        start_time: *start_time,
                        progress: elapsed / animation_duration,
                    }
                }
            }

            ContentState::Showing { start_time } => match &self.window_state {
                WindowState::Exiting { .. } => ContentState::Exiting {
                    start_time: current_time,
                    progress: 0.0,
                },
                _ => ContentState::Showing {
                    start_time: *start_time,
                },
            },

            ContentState::Exiting { start_time, .. } => {
                let elapsed = current_time.duration_since(*start_time).as_secs_f32();
                if elapsed >= animation_duration {
                    ContentState::Idle
                } else {
                    ContentState::Exiting {
                        start_time: *start_time,
                        progress: elapsed / animation_duration,
                    }
                }
            }
        };
    }

    pub(super) fn get_animation_progress(&self) -> (f32, f32) {
        let window_progress = match &self.window_state {
            WindowState::Hidden => 0.0,
            WindowState::Entering { progress, .. } => ease_out_cubic(*progress),
            WindowState::Showing { .. } => 1.0,
            WindowState::Exiting { progress, .. } => 1.0 - ease_out_cubic(*progress),
        };

        let content_progress = match &self.content_state {
            ContentState::Idle => 0.0,
            ContentState::Entering { progress, .. } => ease_out_cubic(*progress),
            ContentState::Showing { .. } => 1.0,
            ContentState::Exiting { progress, .. } => 1.0 - ease_out_cubic(*progress),
        };

        (window_progress, content_progress)
    }
}
