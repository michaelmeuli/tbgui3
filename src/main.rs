use app::{
    settings::settings,
    App,
};

mod app;
mod model;



fn main() -> cosmic::iced::Result {
    cosmic::app::run::<App>(settings(), ())
}
