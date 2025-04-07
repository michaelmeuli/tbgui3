use crate::app::{App, Message};
use cosmic::iced::theme;
use cosmic::widget;
use cosmic::Element;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::widget::{button, text_input};

impl App
where
    Self: cosmic::Application,
{
    pub fn view_raw_sequences(&self) -> Element<Message> {
        let title = cosmic::widget::text("TB-Profiler")
            .width(Length::Fill)
            .size(60)
            .align_x(Horizontal::Center);

        let mut run_controls = widget::row();
        run_controls = run_controls.push(
            widget::button::standard("Run Profiler")
                .on_press(Message::OpenRepositoryUrl)
        );
        let content = widget::column()
            .push(title)
            .push(widget::vertical_space().height(20))
            .push(run_controls)
            .spacing(20)
            .align_x(Horizontal::Center);
        content.into()
    }
}
