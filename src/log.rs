use crate::common::*;
use lightning_invoice::ParseOrSemanticError;
use serde::{Deserialize, Serialize};

pub trait Log {
    fn _err(&self, err: LogErr);

    fn err(&self, err: LogErr) -> ErrLogged {
        self._err(err);
        ErrLogged(Private)
    }
}

#[derive(Debug, Clone)]
pub enum LogErr {
    InvoiceCreateNetwork(String),
    /// ParseOrSemanticError is not currently [de]serializable.
    /// https://github.com/rust-bitcoin/rust-lightning-invoice/issues/30
    InvalidInvoiceCreated(ParseOrSemanticError),
    DbStoreInvoiceDuplicate(Lesser, Invoice),
}

/// This type is not constructable outside this file.
/// It is empty, but serves as proof that an error was logged.
/// Clone is intentionally not implemented becase only one
/// ErrLogged instance should exist per call to log.
#[derive(Debug)]
pub struct ErrLogged(Private);
#[derive(Debug)]
struct Private;

#[derive(Debug)]
pub enum LoggedOr<T> {
    Logged(ErrLogged),
    UnLogged(T),
}

impl<T> LoggedOr<T> {
    /// Helper function. Log error to log and return a LoggedOr::Logged
    pub fn log<L: Log>(log: &L, err: LogErr) -> LoggedOr<T> {
        log.err(err).into()
    }
}

impl<T> From<ErrLogged> for LoggedOr<T> {
    fn from(other: ErrLogged) -> Self {
        LoggedOr::Logged(other)
    }
}

/// Errors may implement this trait when it's possible they are an internal
/// error. Internal errors indicate a problem with the api server, and should
/// not be sent to api clients. Instead, internal errors are logged.
/// In the case of an http REST request, internal errors are logged, then a
/// 500 is sent.
pub trait MaybeServerError {
    /// The type returned when the error is not logged. This type will be sent
    /// to api clients.
    type NotServerError;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError>;
}

pub trait ServerError: Sized {
    fn into_log_err(self) -> LogErr;

    fn log<L: Log>(self, log: &L) -> ErrLogged {
        log.err(self.into_log_err())
    }
}
