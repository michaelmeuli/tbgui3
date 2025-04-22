use crate::app::types::AppError;
use crate::{app::Message, fl};
use cosmic::widget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    Info(AppError),
}

impl DialogPage {
    pub fn view(&self, _text_input_id: &widget::Id) -> widget::Dialog<Message> {
        let _spacing = cosmic::theme::active().cosmic().spacing;

        match self {
            DialogPage::Info(error) => {
                let mut content = widget::column::with_capacity(2).spacing(12);

                match error {
                    AppError::Network(body) => {
                        let title = widget::text::title4("Network error");
                        content = content.push(title);
                        content = content.push(widget::text(body));
                    }
                    AppError::NoItemsChecked(body) => {
                        let title =
                            widget::text::title4("Cannot run tbprofiler with zero items checked");
                        content = content.push(title);
                        content = content.push(widget::text::body(body));
                    }
                    AppError::IO(body) => {
                        let title = widget::text::title4("IO error");
                        content = content.push(title);
                        content = content.push(widget::text(body));
                    }
                }
                widget::dialog()
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(content)
            }
        }
    }
}
