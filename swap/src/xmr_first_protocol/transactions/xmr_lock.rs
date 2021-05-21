use crate::monero::wallet::{TransferRequest, WatchRequest};
use crate::monero::{Amount, PrivateViewKey, Scalar, TransferProof};
use curve25519_dalek::edwards::EdwardsPoint;
use monero::PublicKey;

pub struct XmrLock {
    S_a: monero::PublicKey,
    S_b: monero::PublicKey,
    v_a: PrivateViewKey,
    v_b: PrivateViewKey,
    D: EdwardsPoint,
    amount: Amount,
}

impl XmrLock {
    pub fn new(
        S_a: monero::PublicKey,
        S_b: monero::PublicKey,
        v_a: PrivateViewKey,
        v_b: PrivateViewKey,
        D: EdwardsPoint,
        amount: Amount,
    ) -> Self {
        Self {
            S_a,
            S_b,
            v_a,
            v_b,
            D,
            amount,
        }
    }
    pub fn transfer_request(&self) -> TransferRequest {
        let vk = self.S_a + self.S_b;
        let v = self.v_a + self.v_b;

        // use from KeyGenerator from monero.rs to do the H(vD) bit for the on time
        // address
        let one_time_address = todo!("KeyGenerator.random().one_time_key(self.D)");

        TransferRequest {
            public_spend_key: one_time_address,
            public_view_key: v.public(),
            amount: self.amount,
        }
    }
    pub fn watch_request(&self, transfer_proof: TransferProof) -> WatchRequest {
        let public_spend_key = self.S_a + self.S_b;
        let private_view_key = self.v_a + self.v_b;

        WatchRequest {
            public_spend_key,
            public_view_key: private_view_key.public(),
            transfer_proof,
            conf_target: 1,
            expected: self.amount,
        }
    }
}
