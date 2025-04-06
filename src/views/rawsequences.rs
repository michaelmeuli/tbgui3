use cosmic::widget;
use cosmic::Element;
use crate::app::{App, Message};
use crate::fl;

use cosmic::iced::Length;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::prelude::*;



impl App
where
    Self: cosmic::Application,
{
    pub fn view_raw_sequences(&self) -> Element<Message> {
        widget::text::title1(fl!("welcome"))
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
