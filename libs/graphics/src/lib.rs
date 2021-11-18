#![feature(trivial_bounds)]

mod controls;
mod engine;
pub mod events;
mod gui_trait;
pub mod iced_program_trait;
mod mode;
mod run;
mod state;

pub use engine::{Engine, RenderError};
pub use gui_trait::GuiTrait;
pub use mode::AppMode;
pub use run::event_loop;
pub use state::State;

pub use iced_winit::winit;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
