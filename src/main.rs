use app::{
    settings::settings,
    App,
};

mod app;
mod config;


fn main() -> cosmic::iced::Result {
    cosmic::app::run::<App>(settings(), ())
}
