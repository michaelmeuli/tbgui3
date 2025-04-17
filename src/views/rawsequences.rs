use crate::app::{App, Message};
use cosmic::theme;
use cosmic::widget;
use cosmic::Element;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::widget::{button, text_input};

use cosmic::iced::Alignment;
use cosmic::Apply;


use crate::{
    app::icon_cache,
    fl,
};
use crate::app::icon_cache::get_icon;



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
        run_controls = run_controls
            .push(widget::button::standard("Run Profiler").on_press(Message::RunTbProfiler));
        let content = widget::column()
            .push(title)
            .push(widget::vertical_space().height(20))
            .push(run_controls)
            .spacing(20)
            .align_x(Horizontal::Center);
        content.into()
    }

    pub fn view_raw_sequences2(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.items.is_empty() {
            return widget::container(
                widget::column::with_children(vec![
                    icon_cache::get_icon("applications-office-symbolic", 56).into(),
                    widget::text::title1(fl!("no-list-selected")).into(),
                    widget::text(fl!("no-list-suggestion")).into(),
                ])
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        };

        widget::column::with_capacity(1)
            .push(self.list_view())
            .spacing(spacing.space_xxs)
            .max_width(800.)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    pub fn list_view<'a>(&'a self) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.items.is_empty() {
            return self.empty();
        } else {
            // Provide a default view or handle the non-empty case
            widget::column::with_capacity(1)
                .spacing(spacing.space_xxs)
                .into()
        }
    }

    pub fn empty<'a>(&'a self) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let container = widget::container(
            widget::column::with_children(vec![
                get_icon("task-past-due-symbolic", 56).into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill);

        widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(container)
            .into()
    }

}
