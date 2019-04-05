use crate::common::*;
use futures::future::FutureResult;
use futures::Future;

pub struct ApiLow<D: Db, L: LightningNode> {
    pub database: D,
    pub lighting_node: L,
}

impl<D: Db, L: LightningNode> ApiLow<D, L> {
    pub fn generate_invoice<'a>(
        &'a self,
        lesser: Lesser,
        satoshis: Satoshis,
    ) -> impl Future<Item = Invoice, Error = GenerateInvoiceError> + 'a {
        self.lighting_node
            .create_invoice(satoshis)
            .map_err(GenerateInvoiceError::Create)
            .and_then(move |invoice| {
                // If the database is unable to store the invoice, we don't return it.
                self.database
                    .store_unpaid_invoice(lesser, &invoice)
                    .map_err(GenerateInvoiceError::Store)
                    .map(|()| invoice)
            })
    }

    pub fn pay_invoice<'a>(
        &'a self,
        master: Master,
        invoice: Invoice,
        amount: Satoshis,
        fee: Fee<Satoshis>,
    ) -> impl Future<Item = PaidInvoiceOutgoing, Error = PayInvoiceError> + 'a {
        self.database
            .begin_withdrawal(master, amount, fee)
            .map_err(PayInvoiceError::Begin)
            .and_then(move |()| {
                self.lighting_node
                    .pay_invoice(invoice, amount, fee)
                    .map_err(PayInvoiceError::Pay)
            })
            .and_then(move |paid_invoice| {
                self.database
                    .finish_withdrawal(&paid_invoice)
                    .map(|()| paid_invoice)
                    .map_err(PayInvoiceError::Finish)
            })
        // TODO, if invoice is never paid, refund balance to user account
        // make sure invoice is not paid after balance is refunded
    }

    pub fn check_balance<'a>(
        &'a self,
        middle: Middle,
    ) -> impl Future<Item = Satoshis, Error = CheckBalanceError> + 'a {
        self.database.check_balance(middle)
    }

    pub fn check_invoice_status<'a>(
        &'a self,
        payment_hash: PaymentHash,
    ) -> impl Future<Item = InvoiceStatus, Error = CheckInvoiceStatusError> + 'a {
        self.database.check_invoice_status(payment_hash)
    }
}

#[derive(Debug, Clone)]
pub enum GenerateInvoiceError {
    Create(CreateInvoiceError),
    Store(StoreInvoiceError),
}

#[derive(Debug, Clone)]
pub enum PayInvoiceError {
    /// unpaid_amount + unpaid_fee > MAX
    OverFlow {
        unpaid_invoice: Invoice,
        unpaid_amount: Satoshis,
        unpaid_fee: Fee<Satoshis>,
    },
    Begin(BeginWithdrawalError),
    Pay(PayError),
    Finish(FinishWithdrawalError),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;
    use rand::{thread_rng, Rng};

    fn assert_valid_paid(paid: PaidInvoice, original: Invoice, amount_paid: Satoshis) {
        assert_valid_paid_invoice(paid.clone());
        let PaidInvoice {
            invoice,
            preimage,
            amount_paid: amount_paid_actual,
        } = paid;
        assert_eq!(invoice, original);
        assert_eq!(amount_paid_actual, amount_paid);
    }

    fn assert_valid_paid_invoice(paid: PaidInvoice) {
        let PaidInvoice {
            invoice,
            preimage,
            amount_paid,
        } = paid;
        let amount_requested = Satoshis::from_pico_btc(invoice.amount_pico_btc().unwrap()).unwrap();
        assert!(amount_requested >= amount_paid);
        assert!(amount_requested * Satoshis(2) <= amount_paid);
        assert_eq!(preimage.hash(), get_payment_hash(&invoice));
    }

    fn assert_paid(is: InvoiceStatus) -> PaidInvoice {
        match is {
            InvoiceStatus::Paid(iv) => iv,
            InvoiceStatus::Unpaid => panic!(),
        }
    }

    fn gen_auth() -> (Master, Middle, Lesser) {
        let master: Master = Master::random();
        let middle: Middle = master.into();
        let lesser: Lesser = middle.into();
        (master, middle, lesser)
    }

    fn generate_invoice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        api.generate_invoice(gen_auth().2, Satoshis(1))
            .wait()
            .unwrap();
    }

    fn pay_invoice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let (master, middle, lesser) = gen_auth();
        let invoice = api.generate_invoice(lesser, Satoshis(1)).wait().unwrap();
        api.pay_invoice(master, invoice, Satoshis(1), Fee(Satoshis(10)))
            .wait()
            .unwrap();
    }

    fn check_balance<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let acct_b: Master = gen_auth().0;

        // assert lesser has no balance
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap_err(),
            CheckBalanceError::NoBalance
        );

        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(2))
            .wait()
            .unwrap();

        // pay invoice of n satoshis to lesser
        api.pay_invoice(ACCOUNT_A, invoice, Satoshis(3), DEFAULT_FEE)
            .wait()
            .unwrap();

        // assert lesser has n balance
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(3)
        );
    }

    /// Test behavour of an invoice wit no associated account.
    fn orphan_invoice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let ApiLow {
            lighting_node,
            database,
        } = api;

        // create invoice for n satoshis, this invoice is not yet associated with an account
        let invoice = lighting_node.create_invoice(Satoshis(2)).wait().unwrap();

        // assert invoice status NonExistent
        assert_eq!(
            database
                .check_invoice_status(get_payment_hash(&invoice))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );

        // pay invoice
        lighting_node
            .pay_invoice(invoice.clone(), Satoshis(3), DEFAULT_FEE)
            .wait()
            .unwrap();

        // assert invoice status unpaid
        assert_eq!(
            database
                .check_invoice_status(get_payment_hash(&invoice))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );

        // This test exposes gaps in implementation.
        // When an invoice is paid, how is that payment probagated to the db?
        // What does the db do when it receives an invoice with a previously unknown
        // payment hash.
    }

    fn check_invoice_status<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let acct_b: Master = gen_auth().0;

        // assert lesser balance is Nonexistent
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap_err(),
            CheckBalanceError::NoBalance
        );

        // create invoice for n satoshis
        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(2))
            .wait()
            .unwrap();

        // assert invoice status unpaid
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&invoice))
                .wait()
                .unwrap(),
            InvoiceStatus::Unpaid
        );

        // pay invoice
        api.pay_invoice(ACCOUNT_A, invoice.clone(), Satoshis(3), DEFAULT_FEE)
            .wait()
            .unwrap();

        // assert invoice status paid
        assert_valid_paid(
            assert_paid(
                api.check_invoice_status(get_payment_hash(&invoice))
                    .wait()
                    .unwrap(),
            ),
            invoice,
            Satoshis(3),
        );

        // assert balance for lesser is n
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(3)
        );
    }

    fn check_invoice_status_duo<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        // create {A,B}user,
        let au: Master = gen_auth().0;
        let bu: Master = gen_auth().0;

        // assert {A,B}lesser balance is Nonexistent
        assert_eq!(
            api.check_balance(au.into()).wait().unwrap_err(),
            CheckBalanceError::NoBalance
        );
        assert_eq!(
            api.check_balance(bu.into()).wait().unwrap_err(),
            CheckBalanceError::NoBalance
        );

        // create untracked {A,B}invoice for n satoshis
        let ai = api
            .lighting_node
            .create_invoice(Satoshis(1))
            .wait()
            .unwrap();
        let bi = api
            .lighting_node
            .create_invoice(Satoshis(1))
            .wait()
            .unwrap();

        // assert {A,B}invoice status NonExistent
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&ai))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&bi))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );

        // add Ainvoice to db as unpaid
        api.database
            .store_unpaid_invoice(au.into(), &ai)
            .wait()
            .unwrap();

        // assert Ainvoice status unpaid, Binvoice status NonExistent
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&ai))
                .wait()
                .unwrap(),
            InvoiceStatus::Unpaid
        );
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&bi))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );

        // pay {A,B}invoice
        api.pay_invoice(ACCOUNT_A, ai.clone(), Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        api.pay_invoice(ACCOUNT_A, bi.clone(), Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();

        // assert Ainvoice status paid
        assert_valid_paid(
            assert_paid(
                api.check_invoice_status(get_payment_hash(&ai))
                    .wait()
                    .unwrap(),
            ),
            ai,
            Satoshis(1),
        );
        assert_eq!(
            api.check_invoice_status(get_payment_hash(&bi))
                .wait()
                .unwrap_err(),
            CheckInvoiceStatusError::InvoiceDoesNotExist
        );

        // assert balance for Alesser is n, Blesser is NonExistent
        assert_eq!(api.check_balance(au.into()).wait().unwrap(), Satoshis(1));
        assert_eq!(
            api.check_balance(bu.into()).wait().unwrap_err(),
            CheckBalanceError::NoBalance
        );
    }

    /// pay two separate invoices to the same account, assert correct total balance
    fn pay_two<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let acct_b = Master::random();
        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(1))
            .wait()
            .unwrap();
        api.pay_invoice(ACCOUNT_A, invoice, Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(1)
        );
        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(2))
            .wait()
            .unwrap();
        api.pay_invoice(ACCOUNT_A, invoice, Satoshis(2), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(3)
        );
    }

    // pay the same invoice twice, assert correct total balance
    fn pay_twice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let acct_b = Master::random();
        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(1))
            .wait()
            .unwrap();
        api.pay_invoice(ACCOUNT_A, invoice.clone(), Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(1)
        );
        api.pay_invoice(ACCOUNT_A, invoice, Satoshis(2), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(3)
        );
    }

    fn pay_invoice_to_local<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        // pay N satoshis from account offering M fee
        // get actual fee paid as F
        // assert new_account_balance = old_account_balance - N - F

        let acct_b = Master::random();

        let initial_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        let invoice = api
            .generate_invoice(acct_b.into(), Satoshis(1))
            .wait()
            .unwrap();
        let PaidInvoiceOutgoing {
            paid_invoice,
            fees_offered,
            fees_paid,
        } = api
            .pay_invoice(ACCOUNT_A, invoice, Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(fees_offered, DEFAULT_FEE);
        assert!(fees_paid <= fees_offered);
        assert_valid_paid_invoice(paid_invoice);

        let final_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        assert_eq!(
            initial_a_balance,
            final_a_balance + Satoshis(1) + fees_paid.0
        );
        assert_eq!(
            api.check_balance(acct_b.into()).wait().unwrap(),
            Satoshis(1)
        );
    }

    fn pay_invoice_to_local_to_self<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        // pay N satoshis from account offering M fee
        // get actual fee paid as F
        // assert new_account_balance = old_account_balance - F

        let initial_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        let invoice = api
            .generate_invoice(ACCOUNT_A.into(), Satoshis(1))
            .wait()
            .unwrap();
        let PaidInvoiceOutgoing {
            paid_invoice,
            fees_offered,
            fees_paid,
        } = api
            .pay_invoice(ACCOUNT_A, invoice, Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert_eq!(fees_offered, DEFAULT_FEE);
        assert!(fees_paid <= fees_offered);
        assert_valid_paid_invoice(paid_invoice);

        let final_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        assert_eq!(initial_a_balance, final_a_balance + fees_paid.0);
    }

    fn unused_fees_are_refunded<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let initial_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        let invoice = api
            .generate_invoice(Master::random().into(), Satoshis(1))
            .wait()
            .unwrap();
        let PaidInvoiceOutgoing {
            paid_invoice,
            fees_offered,
            fees_paid,
        } = api
            .pay_invoice(ACCOUNT_A, invoice, Satoshis(1), DEFAULT_FEE)
            .wait()
            .unwrap();
        assert!(
            fees_offered > fees_paid,
            "all fees were used, increase fee offer"
        );
        let final_a_balance = api.check_balance(ACCOUNT_A.into()).wait().unwrap();
        assert_eq!(initial_a_balance, final_a_balance + fees_paid.0);
    }

    /// Create a new test for each constructable combination of db/node implementations
    macro_rules! test_all_impls {
        ($test:ident) => {
            mod $test {
                use super::*;

                #[test]
                fn fake_fake() {
                    $test(ApiLow {
                        database: crate::fake_db::db_with_account_a_balance(),
                        lighting_node: FakeLightningNode {},
                    });
                }

                #[test]
                fn fake_real() {
                    $test(ApiLow {
                        database: crate::fake_db::db_with_account_a_balance(),
                        lighting_node: init_default_lightning_client().unwrap(),
                    });
                }
            }
        };
    }

    test_all_impls!(generate_invoice);
    test_all_impls!(pay_invoice);
    test_all_impls!(check_balance);
    test_all_impls!(orphan_invoice);
    test_all_impls!(check_invoice_status);
    test_all_impls!(check_invoice_status_duo);
    test_all_impls!(pay_two);
    test_all_impls!(pay_twice);
    test_all_impls!(pay_invoice_to_local);
    test_all_impls!(pay_invoice_to_local_to_self);
    test_all_impls!(unused_fees_are_refunded);
}
