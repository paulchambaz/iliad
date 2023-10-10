#[derive(Debug)]
pub enum IliadError {
    Ok,
    InvalidClientConnection,
    DatabaseUnavailable,
    DatabaseNotFound,
    DatabaseAlreadyFound,
    ArchiveFailed,
}
