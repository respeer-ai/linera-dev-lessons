#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    linera_base_types::{WithContractAbi, ChainId},
    views::{RootView, View},
    Contract, ContractRuntime,
};

use fungible_token::{InstantiationArgument, Operation, Message};

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
    type Message = Message;
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
        match operation {
            Operation::Mint { amount } => {
                let application_creator_chain_id = self.runtime.application_creator_chain_id();
                self.send_message(application_creator_chain_id, Message::Mint { amount });
            }
            Operation::Transfer { to, amount } => {
                self.state
                    .transfer(amount)
                    .await
                    .expect("Failed OP: transfer");
                self.send_message(to, Message::Transferred {amount});
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::Mint { amount } => {
                let target_amount = self.state.mint(amount).await.expect("Failed MSG: mint");
                let message_origin_chain_id = self.runtime.message_origin_chain_id().unwrap();
                self.send_message(message_origin_chain_id, Message::Minted { amount: target_amount });
            }
            Message::Minted { amount } => {
                self.state
                    .received(amount)
                    .await
                    .expect("Failed MSG: minted");
            }
            Message::Transferred { amount } => {
                self.state
                    .received(amount)
                    .await
                    .expect("Failed MSG: minted");
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl FungibleTokenContract {
    fn send_message(&mut self, target: ChainId, message: Message) {
        self.runtime
            .prepare_message(message)
            .with_authentication()
            .send_to(target);
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
