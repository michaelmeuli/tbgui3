use crate::app::types::AppError;


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    Info(AppError),
}