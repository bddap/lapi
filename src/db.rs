use crate::common::*;
use futures::future::FutureResult;

pub trait Db {
    fn store_unpaid_invoice(
        &self,
        lesser: &Lesser,
        invoice: &Invoice,
    ) -> FutureResult<(), StoreInvoiceError>;

    fn begin_withdrawal(
        &self,
        master: Master,
        amount: Satoshis,
        fee: Fee<Satoshis>,
    ) -> FutureResult<(), BeginWithdrawalError>;

    fn finish_withdrawal(
        &self,
        invoice: &PaidInvoiceOutgoing,
    ) -> FutureResult<(), FinishWithdrawalError>;

    fn check_balance(&self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError>;

    fn check_invoice_status(
        &self,
        payment_hash: U256,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError>;
}

#[derive(Debug, Clone)]
pub enum StoreInvoiceError {
    /// Invoice has already been stored.
    EntryAlreadyExists(Lesser, Invoice),
}

impl ServerError for StoreInvoiceError {
    fn into_log_err(self) -> LogErr {
        match self {
            StoreInvoiceError::EntryAlreadyExists(lesser, invoice) => {
                LogErr::DbStoreInvoiceDuplicate(lesser, invoice)
            }
        }
    }
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
    WithdrawalNotInProgress(PaidInvoiceOutgoing),
    /// Numeric overflow when refunding unused fees to account
    /// current_balance + refund_amount > MAX
    Overflow {
        account: Lesser,
        current_balance: Satoshis,
        refund_amount: Satoshis,
    },
}

#[derive(Debug, Clone)]
pub enum CheckBalanceError {
    /// The account in question does not exist.
    NoBalance,
}

#[derive(Debug, Clone)]
pub enum CheckInvoiceStatusError {
    /// This invoice was never generated
    InvoiceDoesNotExist,
    /// Network error occured while checking db
    Unknown(String), // TODO, remove this generic error in favor of other, more concrete errors
}
