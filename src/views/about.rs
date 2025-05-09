use crate::app::{Message, Tbgui};
use crate::fl;
use cosmic::iced::Alignment;
use cosmic::widget;
use cosmic::Element;
use cosmic::{cosmic_theme, theme};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/uzh-logo-white.svg");

impl Tbgui
where
    Self: cosmic::Application,
{
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_l, ..
        } = theme::active().cosmic().spacing;
        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));
        let title = widget::text::title3(fl!("app-title"));
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");
        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);
        widget::column()
            .push(widget::vertical_space().height(space_l))
            .push(icon)
            .push(widget::vertical_space().height(space_l))
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }
}
