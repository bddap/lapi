use crate::common::*;
use bitcoin_hashes::{sha256, Hash};
use futures::future::FutureResult;
use lightning_invoice::{Currency, InvoiceBuilder};
use secp256k1::{key::SecretKey, Secp256k1};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct FakeLightningNode {
    preimages: Mutex<BTreeMap<PaymentHash, Preimage>>,
}

impl LightningNode for FakeLightningNode {
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError> {
        let private_key = SecretKey::from_slice(&[
            0xe1, 0x26, 0xf6, 0x8f, 0x7e, 0xaf, 0xcc, 0x8b, 0x74, 0xf5, 0x4d, 0x26, 0x9f, 0xe2,
            0x06, 0xbe, 0x71, 0x50, 0x00, 0xf9, 0x4d, 0xac, 0x06, 0x7d, 0x1c, 0x04, 0xa8, 0xca,
            0x3b, 0x2d, 0xb7, 0x34,
        ])
        .unwrap();
        let random_pre = Preimage(U256::random());
        self.put_preimage(random_pre.clone());
        let payment_hash = sha256::Hash::from_slice(&random_pre.hash().0).unwrap();
        satoshis
            .checked_to_pico_btc()
            .ok_or(CreateInvoiceError::TooLarge)
            .map(|amount_pico_btc| {
                InvoiceBuilder::new(Currency::Bitcoin)
                    .amount_pico_btc(amount_pico_btc)
                    .description("Test invoice. Do not fill.".into())
                    .payment_hash(payment_hash)
                    .current_timestamp()
                    .build_signed(|hash| Secp256k1::new().sign_recoverable(hash, &private_key))
                    .unwrap()
            })
            .into()
    }

    fn pay_invoice(
        &self,
        invoice: Invoice,
        amount: Satoshis,
        max_fee: Fee<Satoshis>,
    ) -> FutureResult<PaidInvoiceOutgoing, PayError> {
        // Yup, looks paid to me.
        let preimage = self.get_preimage(get_payment_hash(&invoice)).unwrap();
        let paid_invoice = PaidInvoice::create(invoice, preimage, amount).unwrap();
        Ok(PaidInvoiceOutgoing {
            paid_invoice,
            fees_offered: max_fee,
            fees_paid: max_fee / Fee(Satoshis(2)),
        })
        .into()
    }
}

impl FakeLightningNode {
    pub fn new() -> Self {
        FakeLightningNode {
            preimages: Mutex::new(BTreeMap::new()),
        }
    }

    fn put_preimage(&self, preimage: Preimage) {
        self.preimages
            .lock()
            .unwrap()
            .insert(preimage.hash(), preimage);
    }

    fn get_preimage(&self, payment_hash: PaymentHash) -> Option<Preimage> {
        self.preimages
            .lock()
            .unwrap()
            .get(&payment_hash)
            .map(|pre| pre.clone())
    }
}
