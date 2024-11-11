use std::time::Instant;

use raqote::*;
use serde::{Deserialize, Serialize};

use crate::components::{Background, Component, Icon, Slider, Text};
use crate::config::{Config, OsdType};
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> + Sized + Sync + Send {
    fn update(&mut self, _: AppMessage) {}
    fn draw(&mut self, ctx: &mut DrawTarget);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AppMessage {
    Close,
    Slider(char, f32),
    Notification(Option<char>, String, Option<String>),
}

pub struct MainApp {
    // Components
    icon: Option<Icon>,
    background: Background,
    slider: Option<Slider>,
    title: Option<Text>,
    description: Option<Text>,

    #[cfg(debug_assertions)]
    debug: Text,

    // Layout properties
    radius: f32,
    half_y: f32,
    config: Config,
    safe_left: f32,
    fg_color: SolidSource,

    // Animation states
    content_state: ContentState,
    window_state: WindowState,
}

enum ContentState {
    Idle,
    Entering { start_time: Instant, progress: f32 },
    Showing { start_time: Instant },
    Exiting { start_time: Instant, progress: f32 },
}

enum WindowState {
    Hidden,
    Entering { start_time: Instant, progress: f32 },
    Showing { start_time: Instant },
    Exiting { start_time: Instant, progress: f32 },
}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let fg_color = config.foreground_color.to_color();
        let radius = config.radius as f32;
        let half_y = config.height as f32 / 2.0;
        let safe_left = (radius * 2.0) - 20.0;

        let background = Background::new(&config, (None, None), ());
        #[cfg(debug_assertions)]
        let debug = Text::new(
            &config,
            (Some(00.0), Some(25.0)),
            (
                0.15,
                3.7,
                "monospace".to_owned(),
                fg_color.clone(),
                String::new(),
            ),
        );

        Self {
            config,
            half_y,
            radius,
            fg_color,
            safe_left,
            background,

            #[cfg(debug_assertions)]
            debug,

            icon: None,
            title: None,
            slider: None,
            description: None,

            content_state: ContentState::Idle,
            window_state: WindowState::Hidden,
        }
    }
}

impl MainApp {
    fn clear_content(&mut self) {
        self.icon = None;
        self.title = None;
        self.slider = None;
        self.description = None;
    }

    fn reset(&mut self) {
        self.clear_content();
        self.content_state = ContentState::Idle;
        self.window_state = WindowState::Hidden;
    }

    fn update_animation_states(&mut self, current_time: Instant) {
        let animation_duration = self.config.animation_duration;
        let show_duration = self.config.show_duration;

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

    fn get_animation_progress(&self) -> (f32, f32) {
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

impl App for MainApp {
    fn update(&mut self, msg: AppMessage) {
        let mut safe_left = self.safe_left;
        let current_time = Instant::now();

        match msg {
            AppMessage::Slider(i, value) => {
                self.clear_content();

                // Actualizar componentes
                if i != '\x00' {
                    self.icon.replace(Icon::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += 40.0;
                }

                self.slider.replace(Slider::new(
                    &self.config,
                    (Some(safe_left), Some(self.half_y)),
                    (value, 4.1),
                ));

                // Manejar estados de animación
                match self.window_state {
                    WindowState::Hidden => {
                        // Si la ventana está oculta, iniciamos ambas animaciones
                        self.window_state = WindowState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                        self.content_state = ContentState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                    }
                    _ => {
                        // Si la ventana ya está visible, solo reiniciamos el contenido
                        self.content_state = ContentState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                        // Reiniciamos el tiempo de showing para la ventana
                        self.window_state = WindowState::Showing {
                            start_time: current_time,
                        };
                    }
                }
            }

            AppMessage::Notification(i, title_content, description) => {
                self.clear_content();

                let mut has_desc = false;
                let mut max_size_text = 3.7;

                if let Some(i) = i {
                    self.icon.replace(Icon::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += 30.0;
                    max_size_text = 4.0;
                }

                if let Some(description) = description {
                    if !description.is_empty() {
                        has_desc = true;
                        self.description.replace(Text::new(
                            &self.config,
                            (Some(safe_left), Some(50.0)),
                            (
                                0.15,
                                max_size_text,
                                "monospace".to_owned(),
                                self.fg_color,
                                description,
                            ),
                        ));
                    }
                }

                self.title.replace(Text::new(
                    &self.config,
                    (
                        Some(safe_left),
                        Some(if has_desc { 30.0 } else { self.half_y }),
                    ),
                    (
                        0.15,
                        max_size_text,
                        "monospace".to_owned(),
                        self.fg_color,
                        title_content,
                    ),
                ));

                // Usar la misma lógica de manejo de estados que en Slider
                match self.window_state {
                    WindowState::Hidden => {
                        self.window_state = WindowState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                        self.content_state = ContentState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                    }
                    _ => {
                        self.content_state = ContentState::Entering {
                            start_time: current_time,
                            progress: 0.0,
                        };
                        self.window_state = WindowState::Showing {
                            start_time: current_time,
                        };
                    }
                }
            }

            AppMessage::Close => {
                self.window_state = WindowState::Exiting {
                    start_time: current_time,
                    progress: 0.0,
                };
                self.content_state = ContentState::Exiting {
                    start_time: current_time,
                    progress: 0.0,
                };
            }
        }
    }

    fn draw(&mut self, ctx: &mut DrawTarget) {
        #[cfg(debug_assertions)]
        self.debug.draw(ctx, 1.0);

        if matches!(self.window_state, WindowState::Hidden)
            && matches!(self.content_state, ContentState::Idle)
        {
            return;
        }

        // Actualizar estados de animación
        self.update_animation_states(Instant::now());

        // Obtener progreso de animaciones
        let (window_progress, content_progress) = self.get_animation_progress();

        // Verificar si debemos resetear
        if matches!(self.window_state, WindowState::Hidden)
            || matches!(self.content_state, ContentState::Idle)
        {
            self.reset();
            return;
        }

        // Dibujar componentes
        #[cfg(debug_assertions)]
        self.debug.change_value(format!(
            "Window: {:?}, Content: {:?}",
            window_progress, content_progress
        ));

        self.background.draw(ctx, window_progress);

        if let Some(slider) = self.slider.as_mut() {
            slider.draw(ctx, content_progress);
        }
        if let Some(icon) = self.icon.as_mut() {
            icon.draw(ctx, content_progress);
        }
        if let Some(title) = self.title.as_mut() {
            title.draw(ctx, content_progress);
        }
        if let Some(description) = self.description.as_mut() {
            description.draw(ctx, content_progress);
        }
    }
}
