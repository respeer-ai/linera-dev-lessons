#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

use fungible_token::{InstantiationArgument, Operation};

use self::state::FungibleTokenState;

pub struct FungibleTokenContract {
    state: FungibleTokenState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(FungibleTokenContract);

impl WithContractAbi for FungibleTokenContract {
    type Abi = fungible_token::FungibleTokenAbi;
}

impl Contract for FungibleTokenContract {
    type Message = ();
    type Parameters = ();
    type InstantiationArgument = InstantiationArgument;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = FungibleTokenState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        FungibleTokenContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        // validate that the application parameters were configured correctly.
        self.runtime.application_parameters();

        self.state.instantiate(argument).await;
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        let owner = self.runtime.authenticated_signer().unwrap();
        match operation {
            Operation::Mint { amount } => {
                self.state
                    .mint(owner, amount)
                    .await
                    .expect("Failed OP: mint");
            }
            Operation::Transfer { to, amount } => {
                self.state
                    .transfer(owner, to, amount)
                    .await
                    .expect("Failed OP: transfer");
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt as _;
    use linera_sdk::{util::BlockingWait, views::View, Contract, ContractRuntime};

    use fungible_token::Operation;

    use super::{FungibleTokenContract, FungibleTokenState};

    #[test]
    fn operation() {
        let initial_value = 10u64;
        let mut app = create_and_instantiate_app(initial_value);

        let increment = 10u64;

        let _response = app
            .execute_operation(Operation::Increment { value: increment })
            .now_or_never()
            .expect("Execution of application operation should not await anything");

        assert_eq!(*app.state.value.get(), initial_value + increment);
    }

    fn create_and_instantiate_app(initial_value: u64) -> FungibleTokenContract {
        let runtime = ContractRuntime::new().with_application_parameters(());
        let mut contract = FungibleTokenContract {
            state: FungibleTokenState::load(runtime.root_view_storage_context())
                .blocking_wait()
                .expect("Failed to read from mock key value store"),
            runtime,
        };

        contract
            .instantiate(initial_value)
            .now_or_never()
            .expect("Initialization of application state should not await anything");

        assert_eq!(*contract.state.value.get(), initial_value);

        contract
    }
}
