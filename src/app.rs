use std::sync::RwLock;
use std::time::Instant;

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use raqote::*;

use crate::components::{Background, Component, Icon, IconComponent, Slider, Text};
use crate::config::{Config, UrgencyConfig};
use crate::notification::Urgency;
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> + Sized + Sync + Send {
    fn show(&self) -> bool;
    fn update(&mut self, _: AppMessage) {}
    fn draw(&mut self, ctx: &mut DrawTarget);
}

#[derive(Debug)]
pub enum AppMessage {
    Close,
    Slider {
        urgency: Urgency,
        icon: Option<Icon>,
        timeout: Option<i32>,
        value: f32,
        bg: Option<String>,
        fg: Option<String>,
    },
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
        let show_duration = config.globals.show_duration.unwrap_or(5.0);
        let window = config.window.clone().unwrap_or_default();
        let radius = window.radius.unwrap_or(100) as f32;
        let half_y = window.height.unwrap_or(80) as f32 / 2.0;
        let safe_left = (radius * 2.0) - 20.0;
        let size = window.height.unwrap_or(80) as f32 * 0.18;
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
        self.show_duration = self.config.globals.show_duration.unwrap_or(5.0);
    }

    fn update_animation_states(&mut self, current_time: Instant) {
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
        let window = self.config.window.clone().unwrap_or_default();

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
                urgency,
                icon,
                timeout,
                value,
                bg,
                fg,
            } => {
                self.clear_content();
                let urgency = UrgencyConfig::from((&self.config, urgency));
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .or_else(|| urgency.show_duration)
                    .or_else(|| self.config.globals.show_duration)
                    .unwrap_or(5.0);

                let fg = fg
                    .or_else(|| urgency.foreground_color.clone())
                    .or_else(|| self.config.globals.foreground_color.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                let bg = bg
                    .or_else(|| urgency.background.clone())
                    .or_else(|| self.config.globals.background.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                self.background.change_color(bg);

                // Actualizar componentes
                if let Some(i) = icon {
                    self.icon.replace(IconComponent::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (fg, i),
                    ));
                    safe_left += self.radius * 0.4;
                }

                if let Some(slider) = self.slider.as_mut() {
                    slider.change_value(value);
                    slider.change_color(bg, fg);
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
                urgency,
                icon: i,
                timeout,
                body: description,
                bg,
                fg,
            } => {
                self.clear_content();
                let urgency = UrgencyConfig::from((&self.config, urgency));
                println!(
                    "Urgency: {urgency:?} - Global: {:?} - Global BG: {:?}",
                    self.config.globals.foreground_color, self.config.globals.background
                );
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .or_else(|| urgency.show_duration)
                    .or_else(|| self.config.globals.show_duration)
                    .unwrap_or(5.0);

                let fg = fg
                    .or_else(|| urgency.foreground_color.clone())
                    .or_else(|| self.config.globals.foreground_color.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                let bg = bg
                    .or_else(|| urgency.background.clone())
                    .or_else(|| self.config.globals.background.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                self.background.change_color(bg);

                let mut has_desc = false;
                let mut max_size_text = 3.7;
                let mut font_size = self.title_text.metrics().font_size;

                if let Some(i) = i {
                    font_size = self.icon_char.metrics().font_size;
                    self.icon.replace(IconComponent::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y - font_size)),
                        (fg, i),
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
                            (
                                Some(safe_left),
                                Some(window.height.unwrap_or(80) as f32 * 0.5),
                            ),
                            (
                                self.description_text.metrics().font_size,
                                self.description_text.layout_runs().map(|l| l.line_w).sum(),
                                max_size_text,
                                fg,
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
                            window.height.unwrap_or(80) as f32 * 0.3
                        } else {
                            self.half_y - (font_size / 2.0)
                        }),
                    ),
                    (
                        self.title_text.metrics().font_size,
                        self.title_text.layout_runs().map(|l| l.line_w).sum(),
                        max_size_text,
                        fg,
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
