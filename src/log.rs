use crate::common::*;
use lightning_invoice::ParseOrSemanticError;
use serde::{Deserialize, Serialize};

pub trait Log: Sync + Send {
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
    InvoicePayUnknown(String), // This variant must be removed before deploying to production.
    FinishWithdrawalError(FinishWithdrawalError),
    /// Invoice was paid, but the hash(preimage) != payment_hash backends such as lnd are expected
    /// to guard against this.
    PayPreimageNoMatch {
        outgoing_paid_invoice: PaidInvoiceOutgoing,
    },
    /// This variant must be removed before deploying to production.
    CheckInvoiceStatusUnknown(String),
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
        LoggedOr::Logged(log.err(err))
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
