use crate::common::*;
use futures::future::FutureResult;
use std::collections::BTreeMap;

pub trait Db {
    fn store_unpaid_invoice(
        &self,
        lesser: Lesser,
        invoice: Invoice,
    ) -> FutureResult<Invoice, StoreInvoiceError>;

    fn begin_withdrawal(
        &self,
        master: Master,
        amount: Satoshis,
    ) -> FutureResult<(), BeginWithdrawalError>;

    fn finish_withdrawal(&self, invoice: PaidInvoice) -> FutureResult<(), FinishWithdrawalError>;

    fn check_balance(&self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError>;

    fn check_invoice_status(
        &self,
        middle: Middle,
        invoice: Invoice,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError>;
}

#[derive(Debug, Clone)]
pub enum StoreInvoiceError {
    /// Invoice has already been stored.
    EntryAlreadyExists,
}

#[derive(Debug, Clone)]
pub enum BeginWithdrawalError {
    /// Not enough funds for withdrawl.
    InsufficeintBalance,
    /// The account in question does not exist.
    NoBalance,
}

#[derive(Debug, Clone)]
pub enum FinishWithdrawalError {
    WithdrawalNotInProgress,
}

#[derive(Debug, Clone)]
pub enum CheckBalanceError {
    /// The account in question does not exist.
    NoBalance,
}

#[derive(Debug, Clone)]
pub enum CheckInvoiceStatusError {
    /// This invoice was never generated for the user in question.
    InvoiceDoesNotExist,
}
