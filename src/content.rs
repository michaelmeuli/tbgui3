use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        task::Handle,
        Alignment, Length, Subscription,
    },
    iced_widget::row,
    theme, widget, Apply, Element,
};

use crate::app::icon_cache::get_icon;
use crate::model::sample::Item;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::{app::icon_cache, fl};

pub struct Content {
    items: SlotMap<DefaultKey, Item>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetItems(Vec<Item>),
}

pub enum Task {
    Get(String),
}

impl Content {
    pub fn new() -> Self {
        Self {
            items: SlotMap::new(),
            task_input_ids: SecondaryMap::new(),
        }
    }

    fn list_header<'a>(&'a self) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let default_icon = emojis::get_by_shortcode("pencil").unwrap().to_string();

        widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
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

    pub fn update(&mut self, message: Message) -> Vec<Task> {
        let mut tasks = Vec::new();
        match message {
            Message::SetItems(tasks) => {
                self.items.clear();
                for task in tasks {
                    let id = self.items.insert(task);
                    self.task_input_ids.insert(id, widget::Id::unique());
                }
            }
        }
        tasks.push(Task::Get("".to_string()));
        tasks
    }

    pub fn view(&self) -> Element<Message> {
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
}
