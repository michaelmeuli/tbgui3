use super::config::{AppError, TbguiConfig};
use super::utils::*;
use crate::model::sample::RemoteState;
use crate::{DEFAULT_TEMPLATE_FILENAME_LOCAL, RESULT_DIR_LOCAL};
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use directories_next::UserDirs; // TODO: Remove this dependency
use rfd::FileDialog; // TODO: Remove this dependency
use russh_sftp::client::fs::ReadDir;
use russh_sftp::client::SftpSession;
use std::fs;
use std::path::PathBuf;
use tokio::fs::create_dir_all;

pub async fn create_client(config: &TbguiConfig) -> Result<Client, AppError> {
    let key_path = UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".ssh")
        .join("id_rsa");
    if !key_path.exists() {
        return Err(AppError::Network(format!(
            "SSH key file not found at path: {:?}",
            key_path
        )));
    }
    let auth_method = AuthMethod::with_key_file(key_path, None);
    let client = Client::connect(
        ("130.60.24.133", 22),
        config.username.as_deref().ok_or_else(|| {
            AppError::Network("Username is not set in the configuration".to_string())
        })?,
        auth_method,
        ServerCheckMethod::NoCheck,
    )
    .await?;
    Ok(client)
}

pub async fn get_raw_reads(client: &Client, config: &TbguiConfig) -> Result<RemoteState, AppError> {
    let remote_raw_dir = config
        .remote_raw_dir
        .as_deref()
        .ok_or_else(|| AppError::Network("Remote raw directory is not set in the configuration".to_string()))?;

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


pub async fn run_tbprofiler(
    client: &Client,
    items_checked: usize,
    samples: String,
    config: &TbguiConfig,
) -> Result<String, async_ssh2_tokio::Error> {
    if items_checked == 0 {
        return Err(async_ssh2_tokio::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Cannot run tbprofiler with zero items checked",
        )));
    }
    let command_run_tbprofiler = format!(
        "sbatch --array 0-{} {} \"{}\" {} {} {}",
        items_checked - 1,
        config.tb_profiler_script.as_str(),
        samples,
        config.remote_raw_dir.as_str(),
        config.remote_out_dir.as_str(),
        config.user_template_remote.as_str(),
    );
    let commandexecutedresult_run_tbprofiler = client.execute(&command_run_tbprofiler).await?;
    if commandexecutedresult_run_tbprofiler.exit_status != 0 {
        return Err(async_ssh2_tokio::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to run tbprofiler: {:?}",
                commandexecutedresult_run_tbprofiler
            ),
        )));
    }
    Ok(commandexecutedresult_run_tbprofiler.stdout)
}

pub async fn download_results(
    client: &Client,
    config: &TbguiConfig,
) -> Result<(), async_ssh2_tokio::Error> {
    let channel = client.get_channel().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;

    let remote_dir = format!("{}/results", config.remote_out_dir);
    let remote_dir: &str = remote_dir.as_str();
    println!(
        "Downloading results from remote directory: {:?}",
        remote_dir
    );
    let default_local_dir = UserDirs::new().unwrap().home_dir().join(RESULT_DIR_LOCAL);
    if !default_local_dir.exists() {
        create_dir_all(&default_local_dir).await?;
    }
    let local_dir: Option<PathBuf> = FileDialog::new()
        .set_title("Select directory to download results")
        .set_directory(UserDirs::new().unwrap().home_dir().join(RESULT_DIR_LOCAL))
        .pick_folder();
    let local_dir = match local_dir {
        Some(dir) => dir,
        None => {
            return Err(async_ssh2_tokio::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No directory selected",
            )));
        }
    };
    check_if_dir_exists(client, remote_dir).await?;
    create_dir_all(local_dir.clone()).await?;
    let entries: ReadDir = sftp.read_dir(remote_dir).await?;
    for entry in entries {
        let file_name = entry.file_name();
        let file_type = entry.file_type();
        let remote_file_path = format!("{}/{}", remote_dir, file_name);
        let local_file_path = local_dir.join(&file_name).clone();

        if file_type.is_file() && (file_name).ends_with(".docx") {
            download_file(&sftp, &remote_file_path, &local_file_path).await?;
        }
    }
    Ok(())
}

pub async fn delete_results(
    client: &Client,
    config: &TbguiConfig,
) -> Result<(), async_ssh2_tokio::Error> {
    let command_checkdir = format!("ls {}", config.remote_out_dir.as_str());
    let commandexecutedresult_checkdir = client.execute(&command_checkdir).await?;
    if commandexecutedresult_checkdir.exit_status != 0 {
        return Err(async_ssh2_tokio::Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No such directory: {:?}", config.remote_out_dir),
        )));
    }
    let command_rm = format!("rm -rf {}", config.remote_out_dir.as_str());
    let commandexecutedresult_rm = client.execute(&command_rm).await?;
    if commandexecutedresult_rm.exit_status != 0 {
        println!(
            "Remote directory may be emmpty. Failed to delete files on remote: {:?}",
            commandexecutedresult_rm
        );
    }
    let directory = UserDirs::new().unwrap().home_dir().join(RESULT_DIR_LOCAL);
    if !directory.is_dir() {
        println!(
            "Directory {RESULT_DIR_LOCAL} does not exist in local home directory: {:?}",
            directory
        );
        return Ok(());
    }
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

pub async fn download_default_template(
    client: &Client,
    config: &TbguiConfig,
) -> Result<(), async_ssh2_tokio::Error> {
    let remote_file_path = config.default_template_remote.as_str();
    let save_directory: Option<PathBuf> = FileDialog::new()
        .set_title("Select directory to save template")
        .set_directory(UserDirs::new().unwrap().home_dir())
        .pick_folder();
    let save_directory = match save_directory {
        Some(dir) => dir,
        None => {
            println!("No directory selected. Download canceled.");
            return Ok(());
        }
    };
    let file_name: Option<String> = FileDialog::new()
        .set_title("Enter Filename for the Template")
        .set_file_name(DEFAULT_TEMPLATE_FILENAME_LOCAL)
        .save_file()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        });
    let file_name = match file_name {
        Some(name) => name,
        None => {
            println!("No filename specified. Download canceled.");
            return Ok(());
        }
    };
    let local_file_path = save_directory.join(file_name);

    let channel = client.get_channel().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;

    download_file(&sftp, remote_file_path, &local_file_path).await?;
    Ok(())
}

pub async fn upload_user_template(
    client: &Client,
    config: &TbguiConfig,
) -> Result<(), async_ssh2_tokio::Error> {
    let remote_file_path = config.user_template_remote.as_str();
    let local_file_path: Option<PathBuf> = FileDialog::new()
        .set_title("Select File to Upload")
        .set_directory(UserDirs::new().unwrap().home_dir())
        .pick_file();
    let local_file_path = match local_file_path {
        Some(path) => path,
        None => {
            println!("No file selected. Upload canceled.");
            return Ok(());
        }
    };

    let channel = client.get_channel().await?;
    channel.request_subsystem(true, "sftp").await?;
    client
        .upload_file(local_file_path, remote_file_path)
        .await?;
    Ok(())
}
