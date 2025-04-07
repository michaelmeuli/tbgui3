use crate::RESULT_DIR_LOCAL;
use super::config::{TbguiConfig, AppError, AppTheme};
use super::App;
use crate::model::sample::Item;
use async_ssh2_tokio::client::Client;
use cosmic::command;
use directories_next::UserDirs;
use russh_sftp::{client::SftpSession, protocol::OpenFlags};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn download_file(
    sftp: &SftpSession,
    remote_file_path: &str,
    local_file_path: &PathBuf,
) -> Result<(), async_ssh2_tokio::Error> {
    let mut remote_file = sftp
        .open_with_flags(remote_file_path, OpenFlags::READ)
        .await?;
    if let Some(parent) = Path::new(local_file_path).parent() {
        create_dir_all(parent).await?;
    }
    let mut local_file = File::create(local_file_path.clone()).await?;
    let mut buffer = [0u8; 4096];

    loop {
        let n = remote_file.read(&mut buffer).await?;
        if n == 0 {
            break; // End of file
        }
        local_file.write_all(&buffer[..n]).await?;
    }
    Ok(())
}

pub async fn check_if_running(
    client: &Client,
    config: &TbguiConfig,
) -> Result<bool, AppError> {
    let command_check_running = match config.username {
        Some(username) => format!("squeue -u {}", username),
        None => return Err(AppError::Username("Username is not set in the configuration".to_string())),
    };
    let commandexecutedresult_check_if_running = client.execute(&command_check_running).await?;
    let running = commandexecutedresult_check_if_running
        .stdout
        .contains(config.username.as_str());
    Ok(running)
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

pub async fn check_if_dir_exists(
    client: &Client,
    remote_raw_dir: &str,
) -> Result<(), async_ssh2_tokio::Error> {
    let command = format!("test -d {} && echo 'exists'", remote_raw_dir);
    let result = client.execute(&command).await.map_err(|e| {
        log_error(&format!(
            "Failed to check if remote directory exists: {:?}",
            e
        ));
        async_ssh2_tokio::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to check if remote directory exists: {:?}", e),
        ))
    })?;
    if result.stdout.trim() != "exists" {
        log_error(&format!(
            "Remote directory does not exist: {:?}",
            remote_raw_dir
        ));
        Err(async_ssh2_tokio::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Remote directory does not exist: {:?}", remote_raw_dir),
        )))
    } else {
        Ok(())
    }
}

pub fn log_error(message: &str) {
    let log_dir = UserDirs::new()
        .expect("Failed to get user directories")
        .home_dir()
        .join(RESULT_DIR_LOCAL);
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).expect("Failed to create RESULT_DIR_LOCAL");
    }
    let error_file = log_dir.join("error.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&error_file)
        .expect("Failed to open or create log file");
    writeln!(file, "{}", message).expect("Failed to write to log file");
}

pub fn delete_log_file() {
    let error_file = UserDirs::new()
        .unwrap()
        .home_dir()
        .join(RESULT_DIR_LOCAL)
        .join("error.log");
    if error_file.exists() && fs::remove_file(&error_file).is_err() {
        log_error(&format!(
            "Failed to delete error log file: {:?}",
            error_file
        ));
    }
}
