use crate::app::icon_cache::get_icon;
use crate::model::{self, status::Status, List, Sample};
use crate::{app::icon_cache, fl};
use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        task::Handle,
        Alignment, Length, Subscription,
    },
    iced_runtime::task,
    iced_widget::row,
    theme, widget, Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

pub struct Content {
    tasks: SlotMap<DefaultKey, Sample>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Complete(DefaultKey, bool),
    SetItems(Vec<Sample>),
}

pub enum TaskMessage {
    Get(String),
    Update(Sample),
}

impl Content {
    pub fn new() -> Self {
        Self {
            tasks: SlotMap::new(),
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

        if self.tasks.is_empty() {
            return self.empty();
        }

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for (id, item) in &self.tasks {
            let item_checkbox = widget::checkbox("", item.status == Status::Completed)
                .on_toggle(move |value| Message::Complete(id, value));

            let task_item_text = widget::text::title1(item.title.clone());

            let row = widget::row::with_capacity(4)
                .align_y(Alignment::Center)
                .spacing(spacing.space_xxs)
                .padding([spacing.space_xxxs, spacing.space_xxs])
                .push(item_checkbox)
                .push(task_item_text);

            items = items.add(row);
        }

        widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(self.list_header())
            .push(items.apply(widget::scrollable))
            .apply(widget::container)
            .height(Length::Shrink)
            .height(Length::Fill)
            .into()
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

    pub fn update(&mut self, message: Message) -> Vec<TaskMessage> {
        let mut tasks = Vec::new();
        match message {
            Message::SetItems(tasks) => {
                self.tasks.clear();
                for task in tasks {
                    let id = self.tasks.insert(task);
                    self.task_input_ids.insert(id, widget::Id::unique());
                }
            }
            Message::Complete(id, complete) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    tasks.push(TaskMessage::Update(task.clone()));
                }
            }
        }
        //tasks.push(Task::Get("".to_string()));
        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.tasks.is_empty() {
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
