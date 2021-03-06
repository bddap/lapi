use crate::common::*;
use futures::future::FutureResult;
use futures::stream::Stream;
use futures::Future;
use std::sync::Arc;
use std::thread;

pub struct ApiLow<D: Db + 'static, L: LightningNode> {
    database: Arc<D>,
    lighting_node: L,
}

impl<D: Db, L: LightningNode> ApiLow<D, L> {
    /// link lightning node to db
    pub fn create(database: D, lighting_node: L) -> ApiLow<D, L> {
        let database = Arc::new(database);

        // spawn a new thread to take paid invoices from lightning node post them to database
        let db2 = database.clone();
        let stream = lighting_node
            .paid_invoices()
            .map(move |paid_invoice| db2.receive_paid_invoice(paid_invoice));
        thread::spawn(|| stream.for_each(|_| FutureResult::from(Ok(()))).wait());

        // TODO log errors from stream and from processing stream

        ApiLow {
            database,
            lighting_node,
        }
    }

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
        // This function is a tangled bundle of future combinators, sorely in need of async await syntax.

        // A malicious client may attempt to send large values for amount and fee, inducing an add overflow.
        // When amount + fee > u64::MAX, we return InsufficientBalance.
        let total_withdrawal = amount
            .checked_add(&fee.0)
            .ok_or(PayInvoiceError::InsufficientBalance);

        FutureResult::from(total_withdrawal)
            .and_then(move |total_withdrawal| {
                self.database
                    .withdraw(master, total_withdrawal)
                    .map_err(PayInvoiceError::from)
                    .map(move |_| total_withdrawal)
            })
            .and_then(move |total_withdrawal| {
                self.lighting_node
                    .pay_invoice(invoice, amount, fee)
                    .or_else(
                        move |payerr| -> Box<
                            Future<Item = PaidInvoiceOutgoing, Error = PayInvoiceError> + Send,
                        > {
                            match payerr {
                                // payment failed, refund entire transaction
                                PayError::PaymentAborted => Box::new(
                                    self.database
                                        .deposit(master.into(), total_withdrawal)
                                        .map_err(PayInvoiceError::Refund)
                                        .and_then(|()| {
                                            Err(PayInvoiceError::Pay(PayError::PaymentAborted))
                                        }),
                                ),
                                other => {
                                    Box::new(FutureResult::from(Err(PayInvoiceError::Pay(other))))
                                }
                            }
                        },
                    )
            })
            .and_then(move |paid_invoice_outgoing| {
                /// payment succeeded, refund any unused fees
                debug_assert_eq!(fee, paid_invoice_outgoing.fees_offered);
                debug_assert!(
                    paid_invoice_outgoing.fees_offered >= paid_invoice_outgoing.fees_paid
                );
                let change: Fee<Satoshis> =
                    paid_invoice_outgoing.fees_offered - paid_invoice_outgoing.fees_paid;
                self.database
                    .deposit(master.into(), change.0)
                    .map(|()| paid_invoice_outgoing)
                    .map_err(PayInvoiceError::RefundFee)
            })
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
    InsufficientBalance,
    Pay(PayError),
    /// Payment failed, but balance was not refuned due to numerical overflow.
    Refund(DepositError),
    /// Payment succeeded, but fee change was not refuned due to numerical overflow.
    RefundFee(DepositError),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    fn assert_valid_paid(paid: PaidInvoice, original: Invoice, amount_paid: Satoshis) {
        assert_eq!(paid.invoice(), &original);
        assert_eq!(&amount_paid, paid.amount_paid());
    }

    fn assert_paid(is: InvoiceStatus) -> PaidInvoice {
        match is {
            InvoiceStatus::Paid(iv) => iv,
            InvoiceStatus::Unpaid(_) => panic!(),
        }
    }

    fn assert_unpaid(is: InvoiceStatus) -> Invoice {
        match is {
            InvoiceStatus::Paid(_) => panic!(),
            InvoiceStatus::Unpaid(iv) => iv,
        }
    }

    fn generate_invoice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        api.generate_invoice(Master::random().into(), Satoshis(1))
            .wait()
            .unwrap();
    }

    fn pay_invoice<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let master = Master::random();
        let invoice = api
            .generate_invoice(master.into(), Satoshis(1))
            .wait()
            .unwrap();
        api.pay_invoice(ACCOUNT_A, invoice, Satoshis(1), Fee(Satoshis(10)))
            .wait()
            .unwrap();
    }

    fn check_balance<D: Db, L: LightningNode>(api: ApiLow<D, L>) {
        let acct_b = Master::random();

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
            api.check_balance(acct_b.into())
                .wait()
                .expect("balance was not updated"),
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
        let acct_b = Master::random();

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
        assert_unpaid(
            api.check_invoice_status(get_payment_hash(&invoice))
                .wait()
                .unwrap(),
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
        let au = Master::random();
        let bu = Master::random();

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
        assert_unpaid(
            api.check_invoice_status(get_payment_hash(&ai))
                .wait()
                .unwrap(),
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
                    $test(ApiLow::create(
                        crate::fake_db::db_with_account_a_balance(),
                        FakeLightningNode::new(),
                    ));
                }

                #[test]
                fn fake_real() {
                    $test(ApiLow::create(
                        crate::fake_db::db_with_account_a_balance(),
                        init_default_lightning_client().unwrap(),
                    ));
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
