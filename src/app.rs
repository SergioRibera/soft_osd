use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::Instant;

use ::services::{Icon, ServiceBroadcast};
use config::{Config, InputAction, InputModifier, NotificationAction, Urgency, UrgencyItemConfig};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use raqote::*;
use winit::dpi::LogicalPosition;
use winit::event::{ButtonSource, FingerId, Modifiers, MouseScrollDelta, WindowEvent};
use winit::keyboard::ModifiersKeyState;

mod event_loop;
mod services;

use crate::components::{Background, Component, IconComponent, Slider, Text};
use crate::utils::ToColor;

use self::event_loop::{ContentState, WindowState};

type Touch = (Option<LogicalPosition<f32>>, Option<LogicalPosition<f32>>);

pub trait App: From<Config> + Sized + Sync + Send {
    fn show(&self) -> bool;
    fn event(&mut self, _: &WindowEvent) {}
    fn update(&mut self, _: AppMessage) {}
    fn get_output(&self) -> Option<String>;
    fn draw(&mut self, ctx: &mut DrawTarget);
}

#[derive(Debug)]
pub enum AppMessage {
    Close,
    Slider {
        id: Option<u32>,
        urgency: Urgency,
        icon: Option<Icon>,
        timeout: Option<i32>,
        value: f32,
        bg: Option<String>,
        fg: Option<String>,
        output: Option<String>,
    },
    Notification {
        id: Option<u32>,
        title: String,
        urgency: Urgency,
        icon: Option<Icon>,
        timeout: Option<i32>,
        body: Option<String>,
        bg: Option<String>,
        fg: Option<String>,
        output: Option<String>,
    },
}

pub struct MainApp {
    broadcast: Option<ServiceBroadcast>,
    notified_levels: HashSet<u8>,
    modifiers: Modifiers,
    current_id: Option<u32>,
    output: Option<String>,
    touches: HashMap<FingerId, Touch>,

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
            broadcast: None,
            notified_levels: HashSet::new(),
            modifiers: Modifiers::default(),
            touches: HashMap::new(),
            current_id: None,
            output: config.output.clone(),

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

impl App for MainApp {
    fn show(&self) -> bool {
        !matches!(self.window_state, WindowState::Hidden)
            || !matches!(self.content_state, ContentState::Idle)
    }

    fn get_output(&self) -> Option<String> {
        self.output.clone()
    }

    fn event(&mut self, event: &WindowEvent) {
        let Some(actions) = self.config.actions.clone() else {
            return;
        };
        let curr_id = self.current_id;
        let modifiers = if self.modifiers.lalt_state() == ModifiersKeyState::Pressed {
            Some(InputModifier::Alt)
        } else if self.modifiers.lcontrol_state() == ModifiersKeyState::Pressed {
            Some(InputModifier::Ctrl)
        } else if self.modifiers.lshift_state() == ModifiersKeyState::Pressed {
            Some(InputModifier::Shift)
        } else {
            None
        };
        match event {
            WindowEvent::PointerButton {
                state,
                position,
                button,
                ..
            } => {
                let position: winit::dpi::LogicalPosition<f32> = position.to_logical(1.0);
                if *state == winit::event::ElementState::Pressed {
                    if let ButtonSource::Touch { finger_id, .. } = button {
                        self.touches.insert(*finger_id, (Some(position), None));
                    }
                    let input_action = match button {
                        ButtonSource::Mouse(button) => match button {
                            winit::event::MouseButton::Left => InputAction::LeftClick,
                            winit::event::MouseButton::Middle => InputAction::MiddleClick,
                            winit::event::MouseButton::Right => InputAction::RightClick,
                            _ => return,
                        },
                        _ => {
                            return;
                        }
                    };
                    if let Some(input_event) = actions.get(&input_action) {
                        if input_event
                            .modifier
                            .zip(modifiers)
                            .is_some_and(|(m, em)| m == em)
                            || input_event.modifier.is_none()
                        {
                            match input_event.action {
                                NotificationAction::Close => {
                                    self.update(AppMessage::Close);
                                }
                                NotificationAction::OpenNotification => {
                                    if let (Some(broadcast), Some(curr_id)) =
                                        (self.broadcast.clone(), curr_id)
                                    {
                                        tokio::spawn(async move {
                                            broadcast
                                                .notify_action::<Self>(curr_id, "default")
                                                .await;
                                        });
                                        self.update(AppMessage::Close)
                                    }
                                }
                            }
                        }
                    }
                }
                if *state == winit::event::ElementState::Released {
                    let ButtonSource::Touch { finger_id, .. } = button else {
                        return;
                    };
                    let Some((Some(start), _end)) = self.touches.get(finger_id) else {
                        return;
                    };
                    let delta_x = (start.x - position.x).abs();
                    let delta_y = start.y - position.y;
                    let input_action = if delta_y > 50.0 && delta_x < 30.0 {
                        InputAction::TouchSwipeUp
                    } else if delta_y < 50.0 && delta_x < 30.0 {
                        InputAction::TouchSwipeDown
                    } else {
                        return;
                    };

                    if let Some(input_event) = actions.get(&input_action) {
                        if input_event
                            .modifier
                            .zip(modifiers)
                            .is_some_and(|(m, em)| m == em)
                            || input_event.modifier.is_none()
                        {
                            match input_event.action {
                                NotificationAction::Close => self.update(AppMessage::Close),
                                NotificationAction::OpenNotification => {
                                    if let (Some(broadcast), Some(curr_id)) =
                                        (self.broadcast.clone(), curr_id)
                                    {
                                        tokio::spawn(async move {
                                            broadcast
                                                .notify_action::<Self>(curr_id, "default")
                                                .await;
                                        });
                                        self.update(AppMessage::Close)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = *modifiers,
            WindowEvent::MouseWheel { delta, .. } => {
                let MouseScrollDelta::LineDelta(_, y) = delta else {
                    return;
                };
                let input_action = if *y > 0.0 {
                    InputAction::ScrollUp
                } else {
                    InputAction::ScrollDown
                };
                if let Some(input_event) = actions.get(&input_action) {
                    if input_event
                        .modifier
                        .zip(modifiers)
                        .is_some_and(|(m, em)| m == em)
                        || input_event.modifier.is_none()
                    {
                        match input_event.action {
                            NotificationAction::Close => self.update(AppMessage::Close),
                            NotificationAction::OpenNotification => {
                                if let (Some(broadcast), Some(curr_id)) =
                                    (self.broadcast.clone(), curr_id)
                                {
                                    tokio::spawn(async move {
                                        broadcast.notify_action::<Self>(curr_id, "default").await;
                                    });
                                    self.update(AppMessage::Close)
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        };
    }

    fn update(&mut self, msg: AppMessage) {
        let mut safe_left = self.safe_left;
        let current_time = Instant::now();
        let window = self.config.window.clone().unwrap_or_default();

        // Manejar estados de animación
        match self.window_state {
            WindowState::Hidden => {
                if matches!(msg, AppMessage::Close) {
                    return;
                }
                self.current_id = None;
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
                id,
                urgency,
                icon,
                timeout,
                value,
                bg,
                fg,
                output,
            } => {
                self.clear_content();
                self.current_id = id;
                self.output = output;

                let mut mult = 3.65;
                let urgency = UrgencyItemConfig::from((&self.config, urgency));
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .or(urgency.show_duration)
                    .or(self.config.globals.show_duration)
                    .unwrap_or(5.0);

                let fg = fg
                    .or(urgency.foreground_color.clone())
                    .or(self.config.globals.foreground_color.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                let bg = bg
                    .or(urgency.background.clone())
                    .or(self.config.globals.background.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                self.background.change_color(bg);

                // Actualizar componentes
                if let Some(i) = icon {
                    self.icon.replace(IconComponent::new(
                        &self.config,
                        (
                            Some(safe_left),
                            Some(self.half_y - (self.icon_char.metrics().font_size / 1.5)),
                        ),
                        (fg, i),
                    ));

                    mult = 4.1;
                    safe_left += self.radius * 0.4;
                }

                if let Some(slider) = self.slider.as_mut() {
                    slider.change_size(window.width.unwrap_or(600) as f32 - (self.radius * mult));
                    slider.change_value(value);
                    slider.change_color(bg, fg);
                } else {
                    self.slider.replace(Slider::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (value, mult),
                    ));
                }
            }

            AppMessage::Notification {
                id,
                title,
                urgency,
                icon: i,
                timeout,
                body: description,
                bg,
                fg,
                output,
            } => {
                self.clear_content();
                self.current_id = id;
                self.output = output;

                let urgency = UrgencyItemConfig::from((&self.config, urgency));
                println!(
                    "Urgency: {urgency:?} - Global: {:?} - Global BG: {:?}",
                    self.config.globals.foreground_color, self.config.globals.background
                );
                self.show_duration = timeout
                    .map(|t| t as f32)
                    .or(urgency.show_duration)
                    .or(self.config.globals.show_duration)
                    .unwrap_or(5.0);

                let fg = fg
                    .or(urgency.foreground_color.clone())
                    .or(self.config.globals.foreground_color.clone())
                    .as_deref()
                    .map(ToColor::to_color)
                    .unwrap();
                let bg = bg
                    .or(urgency.background.clone())
                    .or(self.config.globals.background.clone())
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
