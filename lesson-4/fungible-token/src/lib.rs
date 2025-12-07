use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{Amount, ArithmeticError, ChainId, ContractAbi, ServiceAbi},
    views::ViewError,
};
use num_bigint::BigUint;
use num_traits::{cast::FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct FungibleTokenAbi;

impl ContractAbi for FungibleTokenAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for FungibleTokenAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstantiationArgument {
    pub total_supply: Amount,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub mint_ratio: Amount,
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Mint { amount: Amount },
    Transfer { to: ChainId, amount: Amount },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Mint { amount: Amount },
    Minted { amount: Amount },
    Transferred { amount: Amount },
}

#[derive(Debug, Error)]
pub enum FungibleTokenError {
    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error(transparent)]
    ArithmeticError(#[from] ArithmeticError),

    #[error(transparent)]
    ViewError(#[from] ViewError),
}

pub fn try_amount_mul(amount0: Amount, amount1: Amount) -> Result<Amount, ArithmeticError> {
    let amount0_biguint = BigUint::from_u128(u128::from(amount0)).unwrap();
    let amount1_biguint = BigUint::from_u128(u128::from(amount1)).unwrap();
    let one_biguint = BigUint::from_u128(u128::from(Amount::ONE)).unwrap();

    let result = (&amount0_biguint * &amount1_biguint) / &one_biguint;

    Ok(Amount::from_attos(result.to_u128().unwrap()))
}
