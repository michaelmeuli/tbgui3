use app::{
    settings::{flags, settings},
    App,
};

mod app;
mod content;
mod model;
mod views;

const RESULT_DIR_LOCAL: &str = "tb-profiler-results";
const DEFAULT_TEMPLATE_FILENAME_LOCAL: &str = "default_template.docx";

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<App>(settings(), flags())
}
