use crate::model::list::List;
use crate::app::types::AppError;
use crate::app::config::TbguiConfig;

pub async fn create_list(list: List, config: &mut TbguiConfig) -> Result<List, AppError> {
    config.list = list.clone();
    Err(AppError::Network("create_list() failed".to_string()).into())
}

