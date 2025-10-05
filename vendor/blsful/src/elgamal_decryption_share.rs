use crate::*;

/// A public key share is a point on the curve
/// Must be combined with other public key shares
/// in order to decrypt a ciphertext
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ElGamalDecryptionShare<C: BlsSignatureImpl>(
    #[serde(serialize_with = "traits::public_key_share::serialize::<C, _>")]
    #[serde(deserialize_with = "traits::public_key_share::deserialize::<C, _>")]
    pub <C as Pairing>::PublicKeyShare,
);

impl<C: BlsSignatureImpl> Clone for ElGamalDecryptionShare<C> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<C: BlsSignatureImpl> core::fmt::Debug for ElGamalDecryptionShare<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<C: BlsSignatureImpl> ElGamalDecryptionShare<C> {}

impl<C: BlsSignatureImpl> From<&ElGamalDecryptionShare<C>> for Vec<u8> {
    fn from(value: &ElGamalDecryptionShare<C>) -> Self {
        serde_bare::to_vec(value).expect("failed to serialize ElGamalDecryptionShare")
    }
}

impl<C: BlsSignatureImpl> TryFrom<&[u8]> for ElGamalDecryptionShare<C> {
    type Error = BlsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let share = serde_bare::from_slice(value)?;
        Ok(share)
    }
}

impl_from_derivatives_generic!(ElGamalDecryptionShare);

/// An ElGamal decryption key where the secret key is hidden or combined from shares
/// that can decrypt ciphertext
#[derive(Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElGamalDecryptionKey<C: BlsSignatureImpl>(
    #[serde(serialize_with = "traits::public_key::serialize::<C, _>")]
    #[serde(deserialize_with = "traits::public_key::deserialize::<C, _>")]
    pub <C as Pairing>::PublicKey,
);

impl<C: BlsSignatureImpl> Clone for ElGamalDecryptionKey<C> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<C: BlsSignatureImpl> From<&ElGamalDecryptionKey<C>> for Vec<u8> {
    fn from(value: &ElGamalDecryptionKey<C>) -> Self {
        serde_bare::to_vec(value).expect("failed to serialize ElGamalDecryptionKey")
    }
}

impl<C: BlsSignatureImpl> TryFrom<&[u8]> for ElGamalDecryptionKey<C> {
    type Error = BlsError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let key = serde_bare::from_slice(value)?;
        Ok(key)
    }
}

impl_from_derivatives_generic!(ElGamalDecryptionKey);

impl<C: BlsSignatureImpl> ElGamalDecryptionKey<C> {
    /// Decrypt signcrypt ciphertext
    pub fn decrypt(&self, ciphertext: &ElGamalCiphertext<C>) -> <C as Pairing>::PublicKey {
        ciphertext.c2 - self.0
    }

    /// Combine decryption shares into a signcrypt decryption key
    pub fn from_shares(shares: &[ElGamalDecryptionShare<C>]) -> BlsResult<Self> {
        let points = shares
            .iter()
            .map(|s| s.0)
            .collect::<Vec<<C as Pairing>::PublicKeyShare>>();
        <C as BlsSignatureCore>::core_combine_public_key_shares(&points).map(Self)
    }
}
