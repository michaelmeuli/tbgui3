use cosmic::iced_widget::scrollable::Direction;
use cosmic::iced_widget::scrollable::Scrollbar;
use cosmic::widget;
use cosmic::Element;

use crate::app::{App, Message};
use crate::fl;

use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::iced::alignment::{Horizontal, Vertical};

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
