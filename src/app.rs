pub trait App: Default {
    fn run(&self);
}

#[derive(Default)]
pub struct MainApp;

impl App for MainApp {
    fn run(&self) {
        println!("Renderizando");
    }
}
