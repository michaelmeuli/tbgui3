use crate::model::list::List;
use crate::app::types::AppError;
use crate::app::config::TbguiConfig;
use async_ssh2_tokio::client::Client;
use crate::app::utils::{check_if_dir_exists, log_error};
use crate::model::sample::Item;
use std::collections::HashSet;
use uuid::Uuid;

pub async fn create_list(list: List, config: &mut TbguiConfig) -> Result<List, AppError> {
    config.list = list.clone();
    Err(AppError::Network("create_list() failed".to_string()).into())
}

// pub async fn fetch_tasks(list_id: String, service: TaskService) -> Result<Vec<Item>, AppError> {
//     if let Some(mut service) = service.get_service() {
//         let tasks = service.get_tasks_from_list(list_id).await?;
//         return Ok(tasks);
//     }
//     Ok(vec![])
// }

pub async fn get_raw_reads(
    client: &Client,
    config: &TbguiConfig,
) -> Result<Vec<Item>, AppError> {
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

    Ok(tasks)
}



pub fn create_tasks(reads: Vec<String>) -> Vec<Item> {
    let mut tasks = Vec::new();
    let mut seen_samples = HashSet::new();

    for file_name in reads {
        if let Some((sample, _suffix)) = file_name.split_once('_') {
            if seen_samples.insert(sample.to_string()) {
                tasks.push(Item {
                    id: Uuid::new_v4(),
                    sample: sample.to_string(),
                    is_checked: false,
                });
            }
        }
    }
    tasks
}