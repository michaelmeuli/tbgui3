use cosmic::iced::Length;
use cosmic::widget::{checkbox, text};
use cosmic::Element;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub sample: String,
    pub is_checked: bool,
}

#[derive(Debug, Clone)]
pub enum ItemMessage {
    CheckboxToggled(bool),
}

impl Item {
    pub fn update(&mut self, message: ItemMessage) {
        match message {
            ItemMessage::CheckboxToggled(is_checked) => {
                self.is_checked = is_checked;
            }
        }
    }

    pub fn view(&self) -> Element<ItemMessage> {
        checkbox(&self.sample, self.is_checked)
            .on_toggle(ItemMessage::CheckboxToggled)
            .width(Length::Fill)
            .size(17)
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    All,
    Unchecked,
    Checked,
}

impl Filter {
    pub fn matches(self, item: &Item) -> bool {
        match self {
            Filter::All => true,
            Filter::Unchecked => !item.is_checked,
            Filter::Checked => item.is_checked,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteState {
    pub items: Vec<Item>,
}
