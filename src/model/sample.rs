use crate::app::config::TbguiConfig;
use crate::app::types::{AppError, DialogPage};
use crate::app::utils::{check_if_dir_exists, create_tasks, log_error};
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
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

    pub async fn get_raw_reads(
        client: &Client,
        config: &TbguiConfig,
    ) -> Result<RemoteState, AppError> {
        let remote_raw_dir = config.remote_raw_dir.as_deref().ok_or_else(|| {
            AppError::Network(
                "Remote rawreads directory is not set in the configuration".to_string(),
            )
        })?;

        check_if_dir_exists(client, remote_raw_dir).await?;

        let command = format!("ls {}", remote_raw_dir);
        let result = client.execute(&command).await.map_err(|e| {
            let msg = format!("Failed to list files in remote directory: {:?}", e);
            log_error(&msg);
            AppError::Network(msg)
        })?;

        let raw_reads: Vec<String> = result.stdout.lines().map(String::from).collect();
        let tasks = create_tasks(raw_reads);

        Ok(RemoteState { items: tasks })
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
