use crate::common::*;
use futures::future::FutureResult;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct FakeDb(Mutex<FakeDbInner>);

impl Db for FakeDb {
    fn store_unpaid_invoice(
        &self,
        lesser: Lesser,
        invoice: Invoice,
    ) -> FutureResult<Invoice, StoreInvoiceError> {
        self.0.lock().unwrap().store_unpaid_invoice(lesser, invoice)
    }

    fn begin_withdrawal(
        &self,
        master: Master,
        amount: Satoshis,
    ) -> FutureResult<(), BeginWithdrawalError> {
        self.0.lock().unwrap().begin_withdrawal(master, amount)
    }

    fn finish_withdrawal(&self, invoice: PaidInvoice) -> FutureResult<(), FinishWithdrawalError> {
        self.0.lock().unwrap().finish_withdrawal(invoice)
    }

    fn check_balance(&self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError> {
        self.0.lock().unwrap().check_balance(middle)
    }

    fn check_invoice_status(
        &self,
        middle: Middle,
        invoice: Invoice,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError> {
        self.0.lock().unwrap().check_invoice_status(middle, invoice)
    }
}

struct FakeDbInner {
    balances: BTreeMap<Lesser, Satoshis>,
    history: BTreeMap<Invoice, (Lesser, InvoiceStatus)>,
    withdrawals_in_progress: BTreeMap<Invoice, Lesser>,
}

impl FakeDbInner {
    pub fn store_unpaid_invoice(
        &mut self,
        lesser: Lesser,
        invoice: Invoice,
    ) -> FutureResult<Invoice, StoreInvoiceError> {
        match self
            .history
            .insert(invoice.clone(), (lesser, InvoiceStatus::Unpaid))
        {
            None => {} // Good, there was no entry in the map for this invoice.
            Some(old_value) => {
                // Bad news, we are re-inserting an invoice that was already logged.
                // It should not be possible for this to happen. We have a bug.
                // replace the old value and return an error
                self.history.insert(invoice.clone(), old_value);
                return Err(StoreInvoiceError::EntryAlreadyExists).into();
            }
        }
        Ok(invoice).into()
    }

    pub fn begin_withdrawal(
        &mut self,
        master: Master,
        amount: Satoshis,
    ) -> FutureResult<(), BeginWithdrawalError> {
        let middle: Middle = master.into();
        let lesser: Lesser = middle.into();
        self.balances
            .get_mut(&lesser)
            .ok_or(BeginWithdrawalError::NoBalance)
            .and_then(|balance: &mut Satoshis| {
                let new_balance = balance
                    .checked_sub(&amount)
                    .ok_or(BeginWithdrawalError::InsufficeintBalance)?;
                *balance = new_balance;
                Ok(())
            })
            .into()
    }

    pub fn finish_withdrawal(
        &mut self,
        invoice: PaidInvoice,
    ) -> FutureResult<(), FinishWithdrawalError> {
        self.withdrawals_in_progress
            .remove(&invoice.0)
            .ok_or(FinishWithdrawalError::WithdrawalNotInProgress)
            .map(|_| ())
            .into()
    }

    pub fn check_balance(&mut self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError> {
        self.balances
            .get(&middle.into())
            .map(Clone::clone)
            .ok_or(CheckBalanceError::NoBalance)
            .into()
    }

    pub fn check_invoice_status(
        &mut self,
        middle: Middle,
        invoice: Invoice,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError> {
        let lesser: Lesser = middle.into();
        self.history
            .get(&invoice)
            .and_then(|(entry_lesser, status)| {
                if lesser == *entry_lesser {
                    Some(status)
                } else {
                    // User is looking up an invoice that they do not own. Act like the invoice does not exits.
                    None
                }
            })
            .ok_or(CheckInvoiceStatusError::InvoiceDoesNotExist)
            .map(Clone::clone)
            .into()
    }
}
