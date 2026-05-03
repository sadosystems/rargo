use thiserror::Error;

#[derive(Error, Debug, serde::Deserialize)]
pub enum DecodeError {
    #[error("deserialize fail")]
    Deserialize,
}
