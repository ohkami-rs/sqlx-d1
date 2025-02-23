pub struct D1Error(worker::send::SendWrapper<worker::Error>);

impl From<worker::Error> for D1Error {
    fn from(e: worker::Error) -> Self {
        Self(worker::send::SendWrapper(e))
    }
}
impl D1Error {
    pub(crate) fn from_rust(e: impl std::error::Error) -> Self {
        Self(worker::send::SendWrapper(worker::Error::RustError(e.to_string())))
    }
}

impl std::fmt::Debug for D1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <worker::Error as std::fmt::Debug>::fmt(&self.0.0, f)
    }
}
impl std::fmt::Display for D1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <worker::Error as std::fmt::Display>::fmt(&self.0.0, f)
    }
}
impl std::error::Error for D1Error {}

impl sqlx_core::error::DatabaseError for D1Error {
    fn message(&self) -> &str {
        "Error from D1"
    }

    /* FIXME: match with error message */
    fn kind(&self) -> sqlx_core::error::ErrorKind {
        sqlx_core::error::ErrorKind::Other
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &self.0.0
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        &mut self.0.0
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::new((*self).0.0)
    }
}
