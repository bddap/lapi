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
        fee: Fee<Satoshis>,
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
    history: BTreeMap<PaymentHash, (Lesser, Invoice, InvoiceStatus)>,
    withdrawals_in_progress: BTreeMap<PaymentHash, Withdrawal>,
}

struct Withdrawal {
    // The invoice being paid.
    invoice: Invoice,
    // Sender
    account: Lesser,
    // The fee that we deducted from the senders when starting this witdrawal.
    // Some of this may be refunded later if the actual fee was less than expected.
    fee: Fee<Satoshis>,
}

impl FakeDbInner {
    pub fn store_unpaid_invoice(
        &mut self,
        lesser: Lesser,
        invoice: Invoice,
    ) -> FutureResult<Invoice, StoreInvoiceError> {
        let invoice_uuid = PaymentHash::from_invoice(&invoice);
        match self.history.insert(
            invoice_uuid.clone(),
            (lesser, invoice.clone(), InvoiceStatus::Unpaid),
        ) {
            None => {} // Good, there was no entry in the map for this invoice.
            Some(old_value) => {
                // Bad news, we are re-inserting an invoice that was already logged.
                // It should not be possible for this to happen. We have a bug.
                // replace the old value and return an error
                self.history.insert(invoice_uuid, old_value);
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

    /// Withdawal is confirmed complete.
    pub fn finish_withdrawal(
        &mut self,
        invoice: PaidInvoice,
    ) -> FutureResult<(), FinishWithdrawalError> {
        self.withdrawals_in_progress
            .remove(&PaymentHash::from_invoice(&invoice.invoice))
            .ok_or(FinishWithdrawalError::WithdrawalNotInProgress)
            .map(|withdrawal| {
                // Refund change to account for unused fees
                let change: Fee<Satoshis> = withdrawal.fee - invoice.fees_paid;
            })
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
            .get(&PaymentHash(payment_hash(&invoice)))
            .and_then(|(entry_lesser, _invoice, status)| {
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

    fn add_to_balance(&mut self, lesser: Lesser, amount: Satoshis) -> Result<(), Overflow> {
        if !self.balances.contains_key(&lesser) {
            self.balances.insert(lesser.clone(), Satoshis(0));
        }
        let mut balance = self.balances.get_mut(&lesser).unwrap();
        let new_balance = balance.checked_add(&amount).ok_or(Overflow)?;
        *balance = new_balance;
        debug_assert_eq!(self.balances.get(&lesser).unwrap(), &new_balance);
        Ok(())
    }
}

struct Overflow;

/// Serves as a UUID for an invoice. Used to associate an invoice with a Lesser.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
struct PaymentHash(U256);

impl PaymentHash {
    fn from_invoice(invoice: &Invoice) -> PaymentHash {
        PaymentHash(payment_hash(invoice))
    }
}
