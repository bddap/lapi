use crate::common::*;
use futures::future::FutureResult;

pub trait Db: Sync + Send {
    fn store_unpaid_invoice(
        &self,
        lesser: Lesser,
        invoice: &Invoice,
    ) -> FutureResult<(), StoreInvoiceError>;

    fn withdraw(&self, master: Master, amount: Satoshis) -> FutureResult<(), WithdrawalError>;

    fn deposit(&self, lesser: Lesser, amount: Satoshis) -> FutureResult<(), DepositError>;

    fn check_balance(&self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError>;

    fn check_invoice_status(
        &self,
        payment_hash: U256,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WithdrawalError {
    /// Not enough funds for withdrawl, or account does not exist.
    InsufficeintBalance,
}

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum FinishWithdrawalError {
//     WithdrawalNotInProgress(PaidInvoiceOutgoing),
// }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CheckBalanceError {
    /// The account in question does not exist.
    NoBalance,
}

/// Deposit would cause numeric overflow
/// current_balance + deposit_amount > MAX
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DepositError {
    pub account: Lesser,
    pub current_balance: Satoshis,
    pub deposit_amount: Satoshis,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CheckInvoiceStatusError {
    /// This invoice was never generated
    InvoiceDoesNotExist,
    /// Network error occured while checking db
    Unknown(String), // TODO, remove this generic error in favor of other, more concrete errors
}
