use app::{
    settings::settings,
    App,
};

mod app;
mod model;
mod views;



fn main() -> cosmic::iced::Result {
    cosmic::app::run::<App>(settings(), ())
}
