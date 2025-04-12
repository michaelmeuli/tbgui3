use app::{
    settings::{flags, settings},
    App,
};

mod app;
mod model;
mod views;
mod content;

const RESULT_DIR_LOCAL: &str = "tb-profiler-results";
const DEFAULT_TEMPLATE_FILENAME_LOCAL: &str = "default_template.docx";

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<App>(settings(), flags())
}
