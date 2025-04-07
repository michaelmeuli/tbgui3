use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::{self, nav_bar};
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::app::icon_cache::icon_cache_get;
use crate::app::{App, Message};
use crate::app::Flags;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");



#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NavPage {
    RunTbProfiler,
    DownloadResults,
    DeleteResults,
    Settings,
}

impl NavPage {
    pub fn all() -> &'static [Self] {
        &[
            Self::RunTbProfiler,
            Self::DownloadResults,
            Self::DeleteResults,
            Self::Settings,
        ]
    }

    pub fn title(&self) -> String {
        match self {
            Self::RunTbProfiler => fl!("run-tb-profiler"),
            Self::DownloadResults => fl!("download-results"),
            Self::DeleteResults => fl!("delete-results"),
            Self::Settings => fl!("settings"),
        }
    }

    pub fn icon(&self) -> widget::icon::Icon {
        match self {
            Self::RunTbProfiler => icon_cache_get("play", 16),
            Self::DownloadResults => icon_cache_get("download", 16),
            Self::DeleteResults => icon_cache_get("delete", 16),
            Self::Settings => icon_cache_get("settings", 16),
        }
    }
}




pub fn nav_model(flags: Flags) -> nav_bar::Model {
    let mut nav_model = nav_bar::Model::default();
    for &nav_page in NavPage::all() {
        let id = nav_model
            .insert()
            .icon(nav_page.icon())
            .text(nav_page.title())
            .data::<NavPage>(nav_page)
            .id();
        if nav_page == flags.config.default_page {
            nav_model.activate(id);
        }
    }
    nav_model
}


