#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use anyhow::{bail, Result};
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use hash_edwards_to_edwards::hash_point_to_point;
use rand::{CryptoRng, Rng};
use std::convert::TryInto;
use tiny_keccak::Hasher;

pub const RING_SIZE: usize = 11;
const HASH_KEY_CLSAG_AGG_0: &str = "CLSAG_agg_0";
const HASH_KEY_CLSAG_AGG_1: &str = "CLSAG_agg_1";
const HASH_KEY_CLSAG_ROUND: &str = "CLSAG_round";

// for every iteration we compute:
// c_p = h_prev * mu_P; and
// c_c = h_prev * mu_C.
//
// L_i = s_i * G + c_p * pk_i + c_c * (commitment_i - pseudoutcommitment)
// R_i = s_i * H_p_pk_i + c_p * I + c_c * (z * hash_to_point(signing pk))
//
// h = keccak256("CLSAG_round" || ring
//     ring of commitments || pseudooutput commitment || msg || L_i || R_i)

struct AggregationHashes {
    mu_P: Scalar,
    mu_C: Scalar,
}

impl AggregationHashes {
    pub fn new(
        ring: [EdwardsPoint; RING_SIZE],
        commitment_ring: [EdwardsPoint; RING_SIZE],
        I: EdwardsPoint,
        z: Scalar,
        H_p_pk: EdwardsPoint,
        pseudo_output_commitment: EdwardsPoint,
    ) -> Self {
        let z_key_image = z * H_p_pk;

        let ring = ring
            .iter()
            .flat_map(|pk| pk.compress().as_bytes().to_vec())
            .collect::<Vec<u8>>();
        let commitment_ring = commitment_ring
            .iter()
            .flat_map(|pk| pk.compress().as_bytes().to_vec())
            .collect::<Vec<u8>>();
        let I = I.compress();
        let z_key_image = z_key_image.compress();
        let pseudo_output_commitment = pseudo_output_commitment.compress();

        let mu_P = Self::hash(
            HASH_KEY_CLSAG_AGG_0,
            &ring,
            &commitment_ring,
            &I,
            &z_key_image,
            &pseudo_output_commitment,
        );
        let mu_C = Self::hash(
            HASH_KEY_CLSAG_AGG_1,
            &ring,
            &commitment_ring,
            &I,
            &z_key_image,
            &pseudo_output_commitment,
        );

        Self { mu_P, mu_C }
    }

    // aggregation hashes:
    // mu_{P, C} =
    // keccak256("CLSAG_agg_{0, 1}" ||
    //     ring || ring of commitments || I || z * hash_to_point(signing pk) ||
    // pseudooutput commitment)
    //
    // where z = blinding of real commitment - blinding of pseudooutput commitment.
    fn hash(
        domain_prefix: &str,
        ring: &[u8],
        commitment_ring: &[u8],
        I: &CompressedEdwardsY,
        z_key_image: &CompressedEdwardsY,
        pseudo_output_commitment: &CompressedEdwardsY,
    ) -> Scalar {
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(domain_prefix.as_bytes());
        hasher.update(ring);
        hasher.update(commitment_ring);
        hasher.update(I.as_bytes());
        hasher.update(z_key_image.as_bytes());
        hasher.update(pseudo_output_commitment.as_bytes());

        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);

        Scalar::from_bytes_mod_order(hash)
    }
}

fn challenge(
    prefix: &[u8],
    s_i: Scalar,
    pk_i: EdwardsPoint,
    h_prev: Scalar,
    I: EdwardsPoint,
) -> Result<Scalar> {
    let L_i = s_i * ED25519_BASEPOINT_POINT + h_prev * pk_i;

    let H_p_pk_i = hash_point_to_point(pk_i);
    let R_i = s_i * H_p_pk_i + h_prev * I;

    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(prefix);
    hasher.update(&L_i.compress().as_bytes().to_vec());
    hasher.update(&R_i.compress().as_bytes().to_vec());

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    Ok(Scalar::from_bytes_mod_order(output))
}

// h_0 = keccak256("CLSAG_round" || ring ||
//     ring of commitments || pseudooutput commitment || msg || alpha * G ||
// alpha * hash_to_point(signing pk))
//
// where alpha is random

// TODO: Create ring newtype
fn clsag_round_hash_prefix(
    ring: &[u8],
    commitment_ring: &[u8],
    pseudo_output_commitment: &EdwardsPoint,
    msg: &[u8],
) -> Vec<u8> {
    // TODO: Set capacity
    let mut prefix = Vec::new();

    prefix.extend(HASH_KEY_CLSAG_ROUND.as_bytes());
    prefix.extend(ring);
    prefix.extend(commitment_ring);
    prefix.extend(pseudo_output_commitment.compress().as_bytes());
    prefix.extend(msg);

    prefix
}

#[allow(clippy::too_many_arguments)]
fn final_challenge(
    fake_responses: [Scalar; RING_SIZE - 1],
    ring: [EdwardsPoint; RING_SIZE],
    T_a: EdwardsPoint,
    T_b: EdwardsPoint,
    R_a: EdwardsPoint,
    I_hat_a: EdwardsPoint,
    I_hat_b: EdwardsPoint,
    R_prime_a: EdwardsPoint,
    I: EdwardsPoint,
    msg: &[u8],
) -> Result<(Scalar, Scalar)> {
    let ring_concat = ring
        .iter()
        .flat_map(|pk| pk.compress().as_bytes().to_vec())
        .collect::<Vec<u8>>();
    let prefix = clsag_round_hash_prefix(&ring_concat, todo!(), todo!(), msg);
    let h_0 = {
        let mut keccak = tiny_keccak::Keccak::v256();
        keccak.update(&prefix);
        keccak.update((T_a + T_b + R_a).compress().as_bytes());
        keccak.update((I_hat_a + I_hat_b + R_prime_a).compress().as_bytes());
        let mut output = [0u8; 64];
        keccak.finalize(&mut output);

        Scalar::from_bytes_mod_order_wide(&output)
    };

    let ring_concat = ring
        .iter()
        .flat_map(|pk| pk.compress().as_bytes().to_vec())
        .collect::<Vec<u8>>();

    let h_last = fake_responses
        .iter()
        .enumerate()
        .fold(h_0, |h_prev, (i, s_i)| {
            let pk_i = ring[i + 1];
            // TODO: Do not unwrap here
            challenge(&prefix, *s_i, pk_i, h_prev, I).unwrap()
        });

    Ok((h_last, h_0))
}

pub struct AdaptorSignature {
    s_0_a: Scalar,
    s_0_b: Scalar,
    fake_responses: [Scalar; RING_SIZE - 1],
    h_0: Scalar,
    /// Key image of the real key in the ring.
    I: EdwardsPoint,
}

impl AdaptorSignature {
    pub fn adapt(self, y: Scalar) -> Signature {
        let r_last = self.s_0_a + self.s_0_b + y;

        let responses = self
            .fake_responses
            .iter()
            .chain([r_last].iter())
            .copied()
            .collect::<Vec<_>>()
            .try_into()
            .expect("correct response size");

        Signature {
            responses,
            h_0: self.h_0,
            I: self.I,
        }
    }
}

pub struct Signature {
    pub responses: [Scalar; RING_SIZE],
    pub h_0: Scalar,
    /// Key image of the real key in the ring.
    pub I: EdwardsPoint,
}

impl Signature {
    #[cfg(test)]
    fn verify(&self, ring: [EdwardsPoint; RING_SIZE], msg: &[u8; 32]) -> Result<bool> {
        let ring_concat = ring
            .iter()
            .flat_map(|pk| pk.compress().as_bytes().to_vec())
            .collect::<Vec<u8>>();

        let mut h = self.h_0;

        for (i, s_i) in self.responses.iter().enumerate() {
            let pk_i = ring[(i + 1) % RING_SIZE];
            h = challenge(
                &clsag_round_hash_prefix(&ring_concat, todo!(), todo!(), msg),
                *s_i,
                pk_i,
                h,
                self.I,
            )?;
        }

        Ok(h == self.h_0)
    }
}

impl From<Signature> for monero::util::ringct::Clsag {
    fn from(from: Signature) -> Self {
        Self {
            s: from
                .responses
                .iter()
                .map(|s| monero::util::ringct::Key { key: s.to_bytes() })
                .collect(),
            c1: monero::util::ringct::Key {
                key: from.h_0.to_bytes(),
            },
            D: monero::util::ringct::Key {
                key: from.I.compress().to_bytes(),
            },
        }
    }
}

pub struct Alice0 {
    // secret index is always 0
    ring: [EdwardsPoint; RING_SIZE],
    fake_responses: [Scalar; RING_SIZE - 1],
    msg: [u8; 32],
    // encryption key
    R_a: EdwardsPoint,
    // R'a = r_a*H_p(p_k) where p_k is the signing public key
    R_prime_a: EdwardsPoint,
    // this is not s_a cos of something to with one-time-address??
    s_prime_a: Scalar,
    // secret value:
    alpha_a: Scalar,
    H_p_pk: EdwardsPoint,
    I_a: EdwardsPoint,
    I_hat_a: EdwardsPoint,
    T_a: EdwardsPoint,
}

impl Alice0 {
    pub fn new(
        ring: [EdwardsPoint; RING_SIZE],
        msg: [u8; 32],
        R_a: EdwardsPoint,
        R_prime_a: EdwardsPoint,
        s_prime_a: Scalar,
        rng: &mut (impl Rng + CryptoRng),
    ) -> Result<Self> {
        let mut fake_responses = [Scalar::zero(); RING_SIZE - 1];
        for response in fake_responses.iter_mut().take(RING_SIZE - 1) {
            *response = Scalar::random(rng);
        }
        let alpha_a = Scalar::random(rng);

        let p_k = ring[0];
        let H_p_pk = hash_point_to_point(p_k);

        let I_a = s_prime_a * H_p_pk;
        let I_hat_a = alpha_a * H_p_pk;
        let T_a = alpha_a * ED25519_BASEPOINT_POINT;

        Ok(Alice0 {
            ring,
            fake_responses,
            msg,
            R_a,
            R_prime_a,
            s_prime_a,
            alpha_a,
            H_p_pk,
            I_a,
            I_hat_a,
            T_a,
        })
    }

    pub fn next_message(&self, rng: &mut (impl Rng + CryptoRng)) -> Message0 {
        Message0 {
            pi_a: DleqProof::new(
                ED25519_BASEPOINT_POINT,
                self.T_a,
                self.H_p_pk,
                self.I_hat_a,
                self.alpha_a,
                rng,
            ),
            c_a: Commitment::new(self.fake_responses, self.I_a, self.I_hat_a, self.T_a),
        }
    }

    pub fn receive(self, msg: Message1) -> Result<Alice1> {
        msg.pi_b
            .verify(ED25519_BASEPOINT_POINT, msg.T_b, self.H_p_pk, msg.I_hat_b)?;

        let (h_last, h_0) = final_challenge(
            self.fake_responses,
            self.ring,
            self.T_a,
            msg.T_b,
            self.R_a,
            self.I_hat_a,
            msg.I_hat_b,
            self.R_prime_a,
            self.I_a + msg.I_b,
            &self.msg,
        )?;

        // TODO: alpha_a - h_last * (mu_P * s_prime_a + mu_C * z)
        let s_0_a = self.alpha_a - h_last * self.s_prime_a;

        Ok(Alice1 {
            fake_responses: self.fake_responses,
            h_0,
            I_b: msg.I_b,
            s_0_a,
            I_a: self.I_a,
            I_hat_a: self.I_hat_a,
            T_a: self.T_a,
        })
    }
}

pub struct Alice1 {
    fake_responses: [Scalar; RING_SIZE - 1],
    I_a: EdwardsPoint,
    I_hat_a: EdwardsPoint,
    T_a: EdwardsPoint,
    h_0: Scalar,
    I_b: EdwardsPoint,
    s_0_a: Scalar,
}

impl Alice1 {
    pub fn next_message(&self) -> Message2 {
        Message2 {
            d_a: Opening::new(self.fake_responses, self.I_a, self.I_hat_a, self.T_a),
            s_0_a: self.s_0_a,
        }
    }

    pub fn receive(self, msg: Message3) -> Alice2 {
        let adaptor_sig = AdaptorSignature {
            s_0_a: self.s_0_a,
            s_0_b: msg.s_0_b,
            fake_responses: self.fake_responses,
            h_0: self.h_0,
            I: self.I_a + self.I_b,
        };

        Alice2 { adaptor_sig }
    }
}

pub struct Alice2 {
    pub adaptor_sig: AdaptorSignature,
}

pub struct Bob0 {
    // secret index is always 0
    ring: [EdwardsPoint; RING_SIZE],
    msg: [u8; 32],
    // encryption key
    R_a: EdwardsPoint,
    // R'a = r_a*H_p(p_k) where p_k is the signing public key
    R_prime_a: EdwardsPoint,
    s_b: Scalar,
    // secret value:
    alpha_b: Scalar,
    H_p_pk: EdwardsPoint,
    I_b: EdwardsPoint,
    I_hat_b: EdwardsPoint,
    T_b: EdwardsPoint,
}

impl Bob0 {
    pub fn new(
        ring: [EdwardsPoint; RING_SIZE],
        msg: [u8; 32],
        R_a: EdwardsPoint,
        R_prime_a: EdwardsPoint,
        s_b: Scalar,
        rng: &mut (impl Rng + CryptoRng),
    ) -> Result<Self> {
        let alpha_b = Scalar::random(rng);

        let p_k = ring[0];
        let H_p_pk = hash_point_to_point(p_k);

        let I_b = s_b * H_p_pk;
        let I_hat_b = alpha_b * H_p_pk;
        let T_b = alpha_b * ED25519_BASEPOINT_POINT;

        Ok(Bob0 {
            ring,
            msg,
            R_a,
            R_prime_a,
            s_b,
            alpha_b,
            H_p_pk,
            I_b,
            I_hat_b,
            T_b,
        })
    }

    pub fn receive(self, msg: Message0) -> Bob1 {
        Bob1 {
            ring: self.ring,
            msg: self.msg,
            R_a: self.R_a,
            R_prime_a: self.R_prime_a,
            s_b: self.s_b,
            alpha_b: self.alpha_b,
            H_p_pk: self.H_p_pk,
            I_b: self.I_b,
            I_hat_b: self.I_hat_b,
            T_b: self.T_b,
            pi_a: msg.pi_a,
            c_a: msg.c_a,
        }
    }
}

pub struct Bob1 {
    // secret index is always 0
    ring: [EdwardsPoint; RING_SIZE],
    msg: [u8; 32],
    // encryption key
    R_a: EdwardsPoint,
    // R'a = r_a*H_p(p_k) where p_k is the signing public key
    R_prime_a: EdwardsPoint,
    s_b: Scalar,
    // secret value:
    alpha_b: Scalar,
    H_p_pk: EdwardsPoint,
    I_b: EdwardsPoint,
    I_hat_b: EdwardsPoint,
    T_b: EdwardsPoint,
    pi_a: DleqProof,
    c_a: Commitment,
}

impl Bob1 {
    pub fn next_message(&self, rng: &mut (impl Rng + CryptoRng)) -> Message1 {
        Message1 {
            I_b: self.I_b,
            T_b: self.T_b,
            I_hat_b: self.I_hat_b,
            pi_b: DleqProof::new(
                ED25519_BASEPOINT_POINT,
                self.T_b,
                self.H_p_pk,
                self.I_hat_b,
                self.alpha_b,
                rng,
            ),
        }
    }

    pub fn receive(self, msg: Message2) -> Result<Bob2> {
        let (fake_responses, I_a, I_hat_a, T_a) = msg.d_a.open(self.c_a)?;

        self.pi_a
            .verify(ED25519_BASEPOINT_POINT, T_a, self.H_p_pk, I_hat_a)?;

        let (h_last, h_0) = final_challenge(
            fake_responses,
            self.ring,
            T_a,
            self.T_b,
            self.R_a,
            I_hat_a,
            self.I_hat_b,
            self.R_prime_a,
            I_a + self.I_b,
            &self.msg,
        )?;

        // TODO: alpha_b - h_last * (mu_P * s_b + mu_C * z);
        let s_0_b = self.alpha_b - h_last * self.s_b;

        let adaptor_sig = AdaptorSignature {
            s_0_a: msg.s_0_a,
            s_0_b,
            fake_responses,
            h_0,
            I: I_a + self.I_b,
        };

        Ok(Bob2 { s_0_b, adaptor_sig })
    }
}

pub struct Bob2 {
    s_0_b: Scalar,
    pub adaptor_sig: AdaptorSignature,
}

impl Bob2 {
    pub fn next_message(&self) -> Message3 {
        Message3 { s_0_b: self.s_0_b }
    }
}

struct DleqProof {
    s: Scalar,
    c: Scalar,
}

impl DleqProof {
    fn new(
        G: EdwardsPoint,
        xG: EdwardsPoint,
        H: EdwardsPoint,
        xH: EdwardsPoint,
        x: Scalar,
        rng: &mut (impl Rng + CryptoRng),
    ) -> Self {
        let r = Scalar::random(rng);
        let rG = r * G;
        let rH = r * H;

        let mut keccak = tiny_keccak::Keccak::v256();
        keccak.update(G.compress().as_bytes());
        keccak.update(xG.compress().as_bytes());
        keccak.update(H.compress().as_bytes());
        keccak.update(xH.compress().as_bytes());
        keccak.update(rG.compress().as_bytes());
        keccak.update(rH.compress().as_bytes());

        let mut output = [0u8; 32];
        keccak.finalize(&mut output);

        let c = Scalar::from_bytes_mod_order(output);

        let s = r + c * x;

        Self { s, c }
    }

    fn verify(
        &self,
        G: EdwardsPoint,
        xG: EdwardsPoint,
        H: EdwardsPoint,
        xH: EdwardsPoint,
    ) -> Result<()> {
        let s = self.s;
        let c = self.c;

        let rG = (s * G) + (-c * xG);
        let rH = (s * H) + (-c * xH);

        let mut keccak = tiny_keccak::Keccak::v256();
        keccak.update(G.compress().as_bytes());
        keccak.update(xG.compress().as_bytes());
        keccak.update(H.compress().as_bytes());
        keccak.update(xH.compress().as_bytes());
        keccak.update(rG.compress().as_bytes());
        keccak.update(rH.compress().as_bytes());

        let mut output = [0u8; 32];
        keccak.finalize(&mut output);

        let c_prime = Scalar::from_bytes_mod_order(output);

        if c != c_prime {
            bail!("invalid DLEQ proof")
        }

        Ok(())
    }
}

#[derive(PartialEq)]
struct Commitment([u8; 32]);

impl Commitment {
    fn new(
        fake_responses: [Scalar; RING_SIZE - 1],
        I_a: EdwardsPoint,
        I_hat_a: EdwardsPoint,
        T_a: EdwardsPoint,
    ) -> Self {
        let fake_responses = fake_responses
            .iter()
            .flat_map(|r| r.as_bytes().to_vec())
            .collect::<Vec<u8>>();

        let mut keccak = tiny_keccak::Keccak::v256();
        keccak.update(&fake_responses);
        keccak.update(I_a.compress().as_bytes());
        keccak.update(I_hat_a.compress().as_bytes());
        keccak.update(T_a.compress().as_bytes());

        let mut output = [0u8; 32];
        keccak.finalize(&mut output);

        Self(output)
    }
}

struct Opening {
    fake_responses: [Scalar; RING_SIZE - 1],
    I_a: EdwardsPoint,
    I_hat_a: EdwardsPoint,
    T_a: EdwardsPoint,
}

impl Opening {
    fn new(
        fake_responses: [Scalar; RING_SIZE - 1],
        I_a: EdwardsPoint,
        I_hat_a: EdwardsPoint,
        T_a: EdwardsPoint,
    ) -> Self {
        Self {
            fake_responses,
            I_a,
            I_hat_a,
            T_a,
        }
    }

    fn open(
        self,
        commitment: Commitment,
    ) -> Result<(
        [Scalar; RING_SIZE - 1],
        EdwardsPoint,
        EdwardsPoint,
        EdwardsPoint,
    )> {
        let self_commitment =
            Commitment::new(self.fake_responses, self.I_a, self.I_hat_a, self.T_a);

        if self_commitment == commitment {
            Ok((self.fake_responses, self.I_a, self.I_hat_a, self.T_a))
        } else {
            bail!("opening does not match commitment")
        }
    }
}

// Alice Sends this to Bob
pub struct Message0 {
    c_a: Commitment,
    pi_a: DleqProof,
}

// Bob sends this to ALice
pub struct Message1 {
    I_b: EdwardsPoint,
    T_b: EdwardsPoint,
    I_hat_b: EdwardsPoint,
    pi_b: DleqProof,
}

// Alice sends this to Bob
pub struct Message2 {
    d_a: Opening,
    s_0_a: Scalar,
}

// Bob sends this to Alice
#[derive(Clone, Copy)]
pub struct Message3 {
    s_0_b: Scalar,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn sign_and_verify_success() {
        let msg_to_sign = b"hello world, monero is amazing!!";

        let s_prime_a = Scalar::random(&mut OsRng);
        let s_b = Scalar::random(&mut OsRng);

        let pk = (s_prime_a + s_b) * ED25519_BASEPOINT_POINT;

        let (r_a, R_a, R_prime_a) = {
            let r_a = Scalar::random(&mut OsRng);
            let R_a = r_a * ED25519_BASEPOINT_POINT;

            let pk_hashed_to_point = hash_point_to_point(pk);

            let R_prime_a = r_a * pk_hashed_to_point;

            (r_a, R_a, R_prime_a)
        };

        let mut ring = [EdwardsPoint::default(); RING_SIZE];
        ring[0] = pk;

        ring[1..].fill_with(|| {
            let x = Scalar::random(&mut OsRng);

            x * ED25519_BASEPOINT_POINT
        });

        let alice = Alice0::new(ring, *msg_to_sign, R_a, R_prime_a, s_prime_a, &mut OsRng).unwrap();
        let bob = Bob0::new(ring, *msg_to_sign, R_a, R_prime_a, s_b, &mut OsRng).unwrap();

        let msg = alice.next_message(&mut OsRng);
        let bob = bob.receive(msg);

        let msg = bob.next_message(&mut OsRng);
        let alice = alice.receive(msg).unwrap();

        let msg = alice.next_message();
        let bob = bob.receive(msg).unwrap();

        let msg = bob.next_message();
        let alice = alice.receive(msg);

        let sig = alice.adaptor_sig.adapt(r_a);

        assert!(sig.verify(ring, msg_to_sign).unwrap());
    }
}