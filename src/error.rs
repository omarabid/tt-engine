/// Result Alias with LedgerError
pub type Result<T> = std::result::Result<T, LedgerError>;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum LedgerError {
    #[error("Other: {0}")]
    Adhoc(String),

    #[error("{msg}: {source:?}")]
    Compat {
        msg: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<T> From<std::sync::PoisonError<T>> for LedgerError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        LedgerError::Adhoc("PoisonError".to_string())
    }
}

impl<T: 'static + Send + Sync> From<std::sync::mpsc::SendError<T>> for LedgerError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        LedgerError::Compat {
            msg: String::from("SendError Error"),
            source: Box::new(err),
        }
    }
}
