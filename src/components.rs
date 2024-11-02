use raqote::DrawTarget;

use crate::config::Config;

mod background;
mod icon;
mod slider;
mod text;

pub use background::Background;
pub use icon::Icon;
pub use slider::Slider;
pub use text::Text;

/// Trait para componentes renderizables con ciclo de vida
pub trait Component: Sized {
    type Args;

    /// Inicializa el componente con una configuración
    fn new(config: &Config, args: Self::Args) -> Self;

    /// Renderiza el componente en el contexto de dibujo
    ///
    /// # Argumentos
    /// * `ctx` - Contexto de dibujo de Raqote
    fn draw(&mut self, ctx: &mut DrawTarget, progress: f32);
}
