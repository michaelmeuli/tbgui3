use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    theme, Application,
};
use serde::{Deserialize, Serialize};

use super::{Tbgui, NavPage};

pub const CONFIG_VERSION: u64 = 1;

pub const TBGUI_USERNAME: &str = default_env(option_env!("TBGUI_USERNAME"), "mimeul");
pub const REMOTE_RAW_DIR: &str = default_env(
    option_env!("REMOTE_RAW_DIR"),
    "/shares/sander.imm.uzh/MM/PRJEB57919/raw",
);
pub const TB_PROFILER_SCRIPT: &str = default_env(
    option_env!("TB_PROFILER_SCRIPT"),
    "/shares/sander.imm.uzh/MM/PRJEB57919/scripts/tbprofiler.sh",
);
pub const REMOTE_OUT_DIR: &str = default_env(
    option_env!("REMOTE_OUT_DIR"),
    "/shares/sander.imm.uzh/MM/PRJEB57919/out",
);
pub const DEFAULT_TEMPLATE_REMOTE: &str = default_env(
    option_env!("DEFAULT_TEMPLATE_REMOTE"),
    "/shares/sander.imm.uzh/MM/PRJEB57919/tb-profiler-templates/docx/default_template.docx",
);
pub const USER_TEMPLATE_REMOTE: &str = default_env(
    option_env!("USER_TEMPLATE_REMOTE"),
    "/shares/sander.imm.uzh/MM/PRJEB57919/template/user_template.docx",
);

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TbguiConfig {
    pub username: Option<String>,
    pub remote_raw_dir: Option<String>,
    pub tb_profiler_script: Option<String>,
    pub remote_out_dir: Option<String>,
    pub default_template_remote: Option<String>,
    pub user_template_remote: Option<String>,
    pub default_page: NavPage,
    pub app_theme: AppTheme,
}

impl Default for TbguiConfig {
    fn default() -> Self {
        Self {
            username: Some(TBGUI_USERNAME.to_string()),
            remote_raw_dir: Some(REMOTE_RAW_DIR.to_string()),
            tb_profiler_script: Some(TB_PROFILER_SCRIPT.to_string()),
            remote_out_dir: Some(REMOTE_OUT_DIR.to_string()),
            default_template_remote: Some(DEFAULT_TEMPLATE_REMOTE.to_string()),
            user_template_remote: Some(USER_TEMPLATE_REMOTE.to_string()),
            default_page: NavPage::RunTbProfiler,
            app_theme: AppTheme::Light,
        }
    }
}

const fn default_env(v: Option<&'static str>, default: &'static str) -> &'static str {
    match v {
        Some(v) => v,
        None => default,
    }
}

impl TbguiConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(Tbgui::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> TbguiConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                TbguiConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    log::info!("errors loading config: {:?}", errs);

                    config
                })
            }
            None => TbguiConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}
