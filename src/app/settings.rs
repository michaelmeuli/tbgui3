use cosmic::app::Settings;

use super::localize::localize;
use crate::app::Flags;
use crate::app::TbguiConfig;

pub fn settings() -> Settings {
    localize();
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(TbguiConfig::config().app_theme.theme())
        .debug(false)
    // .size_limits(
    //     cosmic::iced::Limits::NONE
    //         .min_width(360.0)
    //         .min_height(180.0))
}

pub fn flags() -> Flags {
    let (config_handler, config) = (TbguiConfig::config_handler(), TbguiConfig::config());

    Flags {
        config_handler,
        config,
    }
}
