use ethers_core::types::Bytes;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_middleware::signer::SignerMiddlewareError;
use ethers_providers::{FromErr, Middleware};
use ethers_signers::Signer;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct MultiSigner<S> {
    pub signers: Vec<S>,
    pub indicies: usize,
}

impl<S> MultiSigner<S>
where
    S: Signer,
{
    pub fn new(signers: Vec<S>) -> MultiSigner<S> {
        let indicies = signers.len();
        MultiSigner { signers, indicies }
    }
}

#[derive(Clone, Debug)]
pub struct OctopusMiddlewear<M, S> {
    pub(crate) inner: M,
    pub(crate) multisigner: MultiSigner<S>,
}

impl<M, S> FromErr<M::Error> for OctopusMiddlewareError<M, S>
where
    M: Middleware,
    S: Signer,
{
    fn from(src: M::Error) -> Self {
        OctopusMiddlewareError::MiddlewareError(src)
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the blockchain
pub enum OctopusMiddlewareError<M: Middleware, S: Signer> {
    #[error("{0}")]
    /// Thrown when the internal call to the signer fails
    SignerError(S::Error),

    #[error("{0}")]
    /// Thrown when an internal middleware errors
    MiddlewareError(M::Error),

    /// Thrown if the `nonce` field is missing
    #[error("no nonce was specified")]
    NonceMissing,
    /// Thrown if the `gas_price` field is missing
    #[error("no gas price was specified")]
    GasPriceMissing,
    /// Thrown if the `gas` field is missing
    #[error("no gas was specified")]
    GasMissing,
    /// Thrown if a signature is requested from a different address
    #[error("specified from address is not signer")]
    WrongSigner,
    /// Thrown if the signer's chain_id is different than the chain_id of the transaction
    #[error("specified chain_id is different than the signer's chain_id")]
    DifferentChainID,
}

impl<M, S> OctopusMiddlewear<M, S>
where
    M: Middleware,
    S: Signer,
{
    pub fn new(inner: M, multisigner: MultiSigner<S>) -> Self {
        OctopusMiddlewear { inner, multisigner }
    }

    async fn sign_transaction(&self, mut tx: TypedTransaction) -> Result<Bytes, OctopusMiddlewareError<M, S>> {
        let chain_id = self.multisigner.signers[0].chain_id();

        match tx.chain_id() {
            Some(id) if id.as_u64() != chain_id => {
                return Err(OctopusMiddlewareError::DifferentChainID)
            }
            None => {
                tx.set_chain_id(chain_id);
            }
            _ => {}
        }

        let signature = self.multisigner.signers[0].sign_transaction(&tx).await.map_err(OctopusMiddlewareError::SignerError)?;

        Ok(tx.rlp_signed(&signature))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
