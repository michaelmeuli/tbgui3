use crate::app::{Message, Tbgui};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::Element;

impl Tbgui
where
    Self: cosmic::Application,
{
    pub fn view_settings(&self) -> Element<Message> {
        widget::text::title1("Settings")
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
