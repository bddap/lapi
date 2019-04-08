use crate::common::*;
use futures::{future::FutureResult, Future};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct FakeDb(Mutex<FakeDbInner>);

impl FakeDb {
    pub fn new() -> FakeDb {
        let inner = FakeDbInner {
            balances: BTreeMap::new(),
            history: BTreeMap::new(),
        };
        FakeDb(Mutex::new(inner))
    }
}

impl Db for FakeDb {
    fn store_unpaid_invoice(
        &self,
        lesser: Lesser,
        invoice: &Invoice,
    ) -> FutureResult<(), StoreInvoiceError> {
        self.0.lock().unwrap().store_unpaid_invoice(lesser, invoice)
    }

    fn withdraw(&self, master: Master, amount: Satoshis) -> FutureResult<(), WithdrawalError> {
        self.0.lock().unwrap().withdraw(master, amount)
    }

    fn deposit(&self, lesser: Lesser, amount: Satoshis) -> FutureResult<(), DepositError> {
        self.0.lock().unwrap().deposit(lesser, amount)
    }

    fn check_balance(&self, middle: Middle) -> FutureResult<Satoshis, CheckBalanceError> {
        self.0.lock().unwrap().check_balance(middle)
    }

    fn check_invoice_status(
        &self,
        payment_hash: U256,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError> {
        self.0.lock().unwrap().check_invoice_status(payment_hash)
    }
}

struct FakeDbInner {
    balances: BTreeMap<Lesser, Satoshis>,
    history: BTreeMap<PaymentHash, (Lesser, Invoice, InvoiceStatus)>,
}

impl FakeDbInner {
    pub fn store_unpaid_invoice(
        &mut self,
        lesser: Lesser,
        invoice: &Invoice,
    ) -> FutureResult<(), StoreInvoiceError> {
        let invoice_uuid = PaymentHash::from_invoice(&invoice);
        match self.history.insert(
            invoice_uuid.clone(),
            (lesser.clone(), invoice.clone(), InvoiceStatus::Unpaid),
        ) {
            None => Ok(()), // Good, there was no entry in the map for this invoice.
            Some(old_value) => {
                // Bad news, we are re-inserting an invoice that was already logged.
                // It should not be possible for this to happen. We have a bug.
                // replace the old value and return an error
                self.history.insert(invoice_uuid, old_value);
                Err(StoreInvoiceError::EntryAlreadyExists(
                    lesser.clone(),
                    invoice.clone(),
                ))
            }
        }
        .into()
    }

    pub fn withdraw(
        &mut self,
        master: Master,
        amount: Satoshis,
    ) -> FutureResult<(), WithdrawalError> {
        self.balances
            .get_mut(&master.into())
            .ok_or(WithdrawalError::InsufficeintBalance)
            .and_then(|balance: &mut Satoshis| {
                let new_balance = balance
                    .checked_sub(&amount)
                    .ok_or(WithdrawalError::InsufficeintBalance)?;
                *balance = new_balance;
                Ok(())
            })
            .into()
    }

    pub fn deposit(&mut self, lesser: Lesser, amount: Satoshis) -> FutureResult<(), DepositError> {
        self._deposit(lesser, amount).into()
    }

    fn _deposit(&mut self, lesser: Lesser, amount: Satoshis) -> Result<(), DepositError> {
        if !self.balances.contains_key(&lesser) {
            self.balances.insert(lesser.clone(), Satoshis(0));
        }
        let mut balance = self.balances.get_mut(&lesser).unwrap();
        let starting_balance = balance.clone();
        let new_balance = starting_balance.checked_add(&amount).ok_or(DepositError {
            account: lesser,
            current_balance: starting_balance,
            deposit_amount: amount,
        })?;
        balance.0 = new_balance.0;
        debug_assert_eq!(self.balances.get(&lesser).unwrap(), &new_balance);
        Ok(()).into()
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
        payment_hash: U256,
    ) -> FutureResult<InvoiceStatus, CheckInvoiceStatusError> {
        self.history
            .get(&PaymentHash(payment_hash))
            .map(|(_entry_lesser, _invoice, status)| status)
            .ok_or(CheckInvoiceStatusError::InvoiceDoesNotExist)
            .map(Clone::clone)
            .into()
    }
}

#[cfg(test)]
/// Create a fake_db with a balance in test_util::ACCOUNT_A
pub fn db_with_account_a_balance() -> FakeDb {
    use crate::test_util::ACCOUNT_A;
    let db = FakeDb::new();
    {
        let mut dbi = db.0.lock().unwrap();
        assert_eq!(
            dbi.check_balance(ACCOUNT_A.into()).wait(),
            Err(CheckBalanceError::NoBalance)
        );
        dbi.deposit(ACCOUNT_A.into(), Satoshis(500)).wait().unwrap();
    }
    db
}

/// Serves as a UUID for an invoice. Used to associate an invoice with a Lesser.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
struct PaymentHash(U256);

impl PaymentHash {
    fn from_invoice(invoice: &Invoice) -> PaymentHash {
        PaymentHash(get_payment_hash(invoice))
    }
}
