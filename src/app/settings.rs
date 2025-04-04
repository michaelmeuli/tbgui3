use cosmic::app::Settings;

use super::localize::localize;


pub fn settings() -> Settings {
    localize();
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        // .size_limits(
        //     cosmic::iced::Limits::NONE
        //         .min_width(360.0)
        //         .min_height(180.0))
}

