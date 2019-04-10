use crate::common::*;
use std::pin::Pin;

pub trait Db: Sync + Send {
    fn store_unpaid_invoice(
        &self,
        lesser: Lesser,
        invoice: &Invoice,
    ) -> DynFut<(), StoreInvoiceError>;

    fn withdraw(&self, master: Master, amount: Satoshis) -> DynFut<(), WithdrawalError>;

    fn deposit(&self, lesser: Lesser, amount: Satoshis) -> DynFut<(), DepositError>;

    fn check_balance(&self, middle: Middle) -> DynFut<Satoshis, CheckBalanceError>;

    fn check_invoice_status(
        &self,
        payment_hash: U256,
    ) -> DynFut<InvoiceStatus, CheckInvoiceStatusError>;

    /// Invoice has been paid,
    fn receive_paid_invoice(&self, paid_invoice: PaidInvoice) -> DynFut<(), ReceivePaidInvoiceErr>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StoreInvoiceError {
    /// Invoice has already been stored.
    EntryAlreadyExists(Lesser, Invoice),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WithdrawalError {
    /// Not enough funds for withdrawl, or account does not exist.
    InsufficeintBalance,
}

// #[derive(Clone, PartialEq, Eq, Debug)]
pub enum ReceivePaidInvoiceErr {
    // invoice was already paid
    Duplicate(PaidInvoice),
    // invoice was untracked, it was not associated with an account
    NoMatch(PaidInvoice),
    // Deposit failed
    Deposit(DepositError),
}

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
}
