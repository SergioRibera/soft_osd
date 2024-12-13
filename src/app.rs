use clap::ValueEnum;
use std::cell::Cell;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use raqote::*;
use serde::{Deserialize, Serialize};

use crate::components::{Background, Component, Icon, IconComponent, Slider, Text};
use crate::config::Config;
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> + Sized + Sync + Send {
    fn show(&self) -> bool;
    fn update(&mut self, _: AppMessage) {}
    fn draw(&mut self, ctx: &mut DrawTarget);
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

impl From<u8> for Urgency {
    fn from(value: u8) -> Self {
        match value {
            0 => Urgency::Low,
            2 => Urgency::Critical,
            _ => Urgency::Normal,
        }
    }
}

impl From<i8> for Urgency {
    fn from(value: i8) -> Self {
        match value {
            0 => Urgency::Low,
            2 => Urgency::Critical,
            _ => Urgency::Normal,
        }
    }
}

impl From<Urgency> for i8 {
    fn from(value: Urgency) -> Self {
        match value {
            Urgency::Low => 0,
            Urgency::Normal => 1,
            Urgency::Critical => 2,
        }
    }
}

impl From<Urgency> for u8 {
    fn from(value: Urgency) -> Self {
        match value {
            Urgency::Low => 0,
            Urgency::Normal => 1,
            Urgency::Critical => 2,
        }
    }
}

#[derive(Debug)]
pub enum AppMessage {
    Close,
    // urgency, value, icon, timeout, bg_color, fg_color
    Slider {
        urgency: Urgency,
        icon: Option<Icon>,
        timeout: Option<i32>,
        value: f32,
        bg: Option<String>,
        fg: Option<String>,
    },
    // title, urgency, icon, timeout, body, bg_color, fg_color
    Notification {
        title: String,
        urgency: Urgency,
        icon: Option<Icon>,
        timeout: Option<i32>,
        body: Option<String>,
        bg: Option<String>,
        fg: Option<String>,
    },
}

pub struct MainApp {
    fonts: FontSystem,
    sw_cache: SwashCache,
    title_text: Buffer,
    description_text: Buffer,
    icon_char: Buffer,

    // Components
    icon: Option<IconComponent>,
    background: Background,
    slider: Option<Slider>,
    title: Option<Text>,
    description: Option<Text>,

    // Layout properties
    radius: f32,
    half_y: f32,
    config: Config,
    safe_left: f32,
    fg_color: SolidSource,

    // Animation states
    content_state: ContentState,
    window_state: WindowState,
    show_duration: f32,
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

pub static ICON_SIZE: RwLock<f32> = RwLock::new(12.0);

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let fg_color = config.foreground_color.to_color();
        let show_duration = config.show_duration;
        let radius = config.radius as f32;
        let half_y = config.height as f32 / 2.0;
        let safe_left = (radius * 2.0) - 20.0;
        let size = config.height as f32 * 0.18;
        *ICON_SIZE.write().unwrap() = size;

        let mut fonts = FontSystem::new();
        let metrics = Metrics::new(size, size);
        let title_text = Buffer::new(&mut fonts, metrics);
        let description_text = Buffer::new(&mut fonts, metrics.scale(0.85));
        let icon_char = Buffer::new(&mut fonts, metrics);

        let background = Background::new(&config, (None, None), ());

        Self {
            fonts,
            icon_char,
            title_text,
            description_text,
            sw_cache: SwashCache::new(),

            config,
            half_y,
            radius,
            fg_color,
            safe_left,
            background,

            icon: None,
            title: None,
            slider: None,
            description: None,

            content_state: ContentState::Idle,
            window_state: WindowState::Hidden,
            show_duration,
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
        self.show_duration = self.config.show_duration;
    }

    fn update_animation_states(&mut self, current_time: Instant) {
        let animation_duration = self.config.animation_duration;
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
    fn show(&self) -> bool {
        if matches!(self.window_state, WindowState::Hidden)
            && matches!(self.content_state, ContentState::Idle)
        {
            return false;
        }

        !matches!(self.window_state, WindowState::Hidden)
            || !matches!(self.content_state, ContentState::Idle)
    }

    fn update(&mut self, msg: AppMessage) {
        let mut safe_left = self.safe_left;
        let current_time = Instant::now();

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
                if self.slider.is_none() {
                    self.content_state = ContentState::Entering {
                        start_time: current_time,
                        progress: 0.0,
                    };
                }
                // Reiniciamos el tiempo de showing para la ventana
                self.window_state = WindowState::Showing {
                    start_time: current_time,
                };
            }
        }

        match msg {
            AppMessage::Slider {
                urgency: _,
                icon,
                timeout,
                value,
                ..
            } => {
                self.clear_content();
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .unwrap_or_else(|| self.config.show_duration);

                // Actualizar componentes
                if let Some(i) = icon {
                    self.icon.replace(IconComponent::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += self.radius * 0.4;
                }

                if let Some(slider) = self.slider.as_mut() {
                    slider.change_value(value);
                } else {
                    self.slider.replace(Slider::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (value, 4.1),
                    ));
                }
            }

            AppMessage::Notification {
                title,
                urgency: _,
                icon: i,
                timeout,
                body: description,
                ..
            } => {
                self.clear_content();
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .unwrap_or_else(|| self.config.show_duration);

                let mut has_desc = false;
                let mut max_size_text = 3.7;
                let mut font_size = self.title_text.metrics().font_size;

                if let Some(i) = i {
                    font_size = self.icon_char.metrics().font_size;
                    self.icon.replace(IconComponent::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y - font_size)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += self.radius * 0.3;
                    max_size_text = 4.0;
                }

                if let Some(description) = description {
                    if !description.is_empty() {
                        has_desc = true;
                        self.description_text.set_text(
                            &mut self.fonts,
                            &description,
                            Attrs::new(),
                            cosmic_text::Shaping::Advanced,
                        );
                        self.description.replace(Text::new(
                            &self.config,
                            (Some(safe_left), Some(self.config.height as f32 * 0.5)),
                            (
                                self.description_text.metrics().font_size,
                                self.description_text.layout_runs().map(|l| l.line_w).sum(),
                                max_size_text,
                                self.fg_color,
                            ),
                        ));
                    }
                }
                self.title_text.set_text(
                    &mut self.fonts,
                    &title,
                    Attrs::new(),
                    cosmic_text::Shaping::Advanced,
                );
                self.title.replace(Text::new(
                    &self.config,
                    (
                        Some(safe_left),
                        Some(if has_desc {
                            self.config.height as f32 * 0.3
                        } else {
                            self.half_y - (font_size / 2.0)
                        }),
                    ),
                    (
                        self.title_text.metrics().font_size,
                        self.title_text.layout_runs().map(|l| l.line_w).sum(),
                        max_size_text,
                        self.fg_color,
                    ),
                ));
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
                self.show_duration = 0.0;
                return;
            }
        }
    }

    fn draw(&mut self, ctx: &mut DrawTarget) {
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
        self.background.draw(ctx, window_progress, ());

        if let Some(slider) = self.slider.as_mut() {
            slider.draw(ctx, content_progress, ());
        }
        if let Some(icon) = self.icon.as_mut() {
            icon.draw(
                ctx,
                content_progress,
                (&mut self.fonts, &mut self.sw_cache, &mut self.icon_char),
            );
        }
        if let Some(title) = self.title.as_mut() {
            title.draw(
                ctx,
                content_progress,
                (&mut self.fonts, &mut self.sw_cache, &self.title_text),
            );
        }
        if let Some(description) = self.description.as_mut() {
            description.draw(
                ctx,
                content_progress,
                (&mut self.fonts, &mut self.sw_cache, &self.description_text),
            );
        }
    }
}
