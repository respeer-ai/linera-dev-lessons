#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Schema};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::WithServiceAbi,
    linera_base_types::{AccountOwner, Amount},
    views::View,
    Service, ServiceRuntime,
};

use fungible_token::Operation;

use self::state::FungibleTokenState;

pub struct FungibleTokenService {
    state: Arc<FungibleTokenState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(FungibleTokenService);

impl WithServiceAbi for FungibleTokenService {
    type Abi = fungible_token::FungibleTokenAbi;
}

impl Service for FungibleTokenService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = FungibleTokenState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        FungibleTokenService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        Schema::build(
            QueryRoot {
                state: self.state.clone(),
            },
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

struct QueryRoot {
    state: Arc<FungibleTokenState>,
}

#[Object]
impl QueryRoot {
    async fn total_supply(&self) -> Amount {
        *self.state.total_supply.get()
    }

    async fn name(&self) -> String {
        self.state.name.get().clone()
    }

    async fn symbol(&self) -> String {
        self.state.symbol.get().clone()
    }

    async fn decimals(&self) -> u8 {
        *self.state.decimals.get()
    }

    async fn mint_ratio(&self) -> Amount {
        *self.state.mint_ratio.get()
    }

    async fn balance(&self) -> Amount {
        *self.state.balance.get()
    }

    async fn balance_of(&self, owner: AccountOwner) -> Amount {
        self.state
            .balances
            .get(&owner)
            .await
            .expect("Failed get balance")
            .unwrap_or(Amount::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::{Request, Response, Value};
    use futures::FutureExt as _;
    use linera_sdk::{util::BlockingWait, views::View, Service, ServiceRuntime};
    use serde_json::json;

    use super::{FungibleTokenService, FungibleTokenState};

    #[test]
    fn query() {
        let value = 60u64;
        let runtime = Arc::new(ServiceRuntime::<FungibleTokenService>::new());
        let mut state = FungibleTokenState::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");
        state.value.set(value);

        let service = FungibleTokenService { state, runtime };
        let request = Request::new("{ value }");

        let response = service
            .handle_query(request)
            .now_or_never()
            .expect("Query should not await anything");

        let expected = Response::new(Value::from_json(json!({"value": 60})).unwrap());

        assert_eq!(response, expected)
    }
}
