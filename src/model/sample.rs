use super::priority::Priority;
use super::status::Status;
use crate::app::config::TbguiConfig;
use crate::app::types::AppError;
use crate::app::utils::{check_if_dir_exists, log_error};
use async_ssh2_tokio::client::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sample {
    pub(crate) id: String,
    pub parent: String,
    pub title: String,
    pub favorite: bool,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub sub_tasks: Vec<Sample>,
    pub tags: Vec<String>,
    pub notes: String,
}

impl Sample {
    pub fn new(title: String, parent: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parent,
            title,
            favorite: false,
            today: false,
            status: Status::NotStarted,
            priority: Priority::Low,
            sub_tasks: vec![],
            tags: vec![],
            notes: String::new(),
        }
    }

    pub async fn get_raw_reads(
        client: &Client,
        config: &TbguiConfig,
    ) -> Result<Vec<Sample>, AppError> {
        println!("Getting paired reads as items");
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
        let tasks = create_sample_tasks(raw_reads);
        println!("Tasks: {:?}", tasks);

        Ok(tasks)
    }
}

pub fn create_sample_tasks(reads: Vec<String>) -> Vec<Sample> {
    let mut tasks = Vec::new();
    let mut seen_samples = HashSet::new();

    for file_name in reads {
        if let Some((sample, _suffix)) = file_name.split_once('_') {
            if seen_samples.insert(sample.to_string()) {
                let task = Sample::new(sample.to_string(), String::new());
                tasks.push(task);
            }
        }
    }
    tasks
}
