use app::{
    settings::{flags, settings},
    Tbgui,
};

mod actions;
mod app;
mod content;
mod context;
mod dialog;
mod model;
mod views;

const RESULT_DIR_LOCAL: &str = "tb-profiler-results";
const DEFAULT_TEMPLATE_FILENAME_LOCAL: &str = "default_template.docx";

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<Tbgui>(settings(), flags())
}
