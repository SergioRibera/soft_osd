use raqote::DrawTarget;

use crate::config::Config;

mod background;

pub use background::Background;

/// Trait para componentes renderizables con ciclo de vida
pub trait Component: Sized {
    /// Inicializa el componente con una configuración
    fn new(config: &Config) -> Self;

    /// Renderiza el componente en el contexto de dibujo
    ///
    /// # Argumentos
    /// * `ctx` - Contexto de dibujo de Raqote
    fn draw(&self, ctx: &mut DrawTarget, progress: f32);
}
