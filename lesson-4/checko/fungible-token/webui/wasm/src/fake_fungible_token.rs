use fungible_token::Operation;
use async_graphql::{Error, Object};
use linera_base::{
    data_types::Amount,
    identifiers::ChainId,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn parse_query(&self) -> u64 {
        0
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn mint(
        &self,
        amount: Amount,
    ) -> Result<Vec<u8>, Error> {
        Ok(bcs::to_bytes(&Operation::Mint { amount })?)
    }

    async fn transfer(
        &self,
        to: ChainId,
        amount: Amount,
    ) -> Result<Vec<u8>, Error> {
        Ok(bcs::to_bytes(&Operation::Transfer { to, amount })?)
    }
}
