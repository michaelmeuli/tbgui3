#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    Info(AppError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppError {
    Network(String),
    NoItemsChecked(String),
    IO(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Network(err) => write!(f, "{}", err),
            AppError::NoItemsChecked(err) => write!(f, "{}", err),
            AppError::IO(err) => write!(f, "{}", err),
        }
    }
}

impl From<async_ssh2_tokio::Error> for AppError {
    fn from(error: async_ssh2_tokio::Error) -> Self {
        AppError::Network(format!("async_ssh2_tokio error: {}", error))
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Network(format!("std::io::Error: {}", error))
    }
}

impl From<russh_sftp::client::error::Error> for AppError {
    fn from(error: russh_sftp::client::error::Error) -> Self {
        AppError::Network(format!("russh_sftp::client::error::Error: {}", error))
    }
}
