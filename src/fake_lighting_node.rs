use crate::common::*;
use bitcoin_hashes::{sha256, Hash};
use futures::future::FutureResult;
use lightning_invoice::{Currency, InvoiceBuilder};
use rand::{thread_rng, Rng};
use secp256k1::{key::SecretKey, Secp256k1};

pub struct FakeLightningNode {}

impl LightningNode for FakeLightningNode {
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError> {
        let private_key = SecretKey::from_slice(&[
            0xe1, 0x26, 0xf6, 0x8f, 0x7e, 0xaf, 0xcc, 0x8b, 0x74, 0xf5, 0x4d, 0x26, 0x9f, 0xe2,
            0x06, 0xbe, 0x71, 0x50, 0x00, 0xf9, 0x4d, 0xac, 0x06, 0x7d, 0x1c, 0x04, 0xa8, 0xca,
            0x3b, 0x2d, 0xb7, 0x34,
        ])
        .unwrap();
        let random: [u8; 32] = thread_rng().gen();
        let payment_hash = sha256::Hash::from_slice(&random).unwrap();
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
        max_fee: Fee<Satoshis>,
    ) -> FutureResult<PaidInvoice, PayError> {
        // Yup, looks paid to me.
        Ok(PaidInvoice {
            invoice,
            preimage: U256::zero(),
            fees_paid: max_fee / Fee(Satoshis(2)),
        })
        .into()
    }
}
