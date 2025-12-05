use linera_sdk::{
    ensure,
    linera_base_types::Amount,
    views::{linera_views, RegisterView, RootView, ViewStorageContext},
};

use fungible_token::{try_amount_mul, FungibleTokenError, InstantiationArgument};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct FungibleTokenState {
    pub total_supply: RegisterView<Amount>,
    pub name: RegisterView<String>,
    pub symbol: RegisterView<String>,
    pub decimals: RegisterView<u8>,
    pub mint_ratio: RegisterView<Amount>,
    pub balance: RegisterView<Amount>,

    pub my_balance: RegisterView<Amount>,
}

#[allow(dead_code)]
impl FungibleTokenState {
    pub(crate) async fn instantiate(&mut self, argument: InstantiationArgument) {
        self.total_supply.set(argument.total_supply);
        self.balance.set(argument.total_supply);
        self.name.set(argument.name);
        self.symbol.set(argument.symbol);
        self.decimals.set(argument.decimals);
        self.mint_ratio.set(argument.mint_ratio);
    }

    pub(crate) async fn mint(
        &mut self,
        amount: Amount,
    ) -> Result<Amount, FungibleTokenError> {
        let target_amount = try_amount_mul(amount, *self.mint_ratio.get())?;
        let total_balance = *self.balance.get();

        if target_amount.gt(&total_balance) {
            return Err(FungibleTokenError::InsufficientFunds);
        }

        self.balance.set(total_balance.try_sub(target_amount)?);

        Ok(target_amount)
    }

    pub(crate) async fn transfer(
        &mut self,
        amount: Amount,
    ) -> Result<(), FungibleTokenError> {
        let from_balance = *self.my_balance.get();

        ensure!(
            from_balance.gt(&amount),
            FungibleTokenError::InsufficientFunds
        );

        self.my_balance.set(from_balance.try_sub(amount)?);

        Ok(())
    }

    pub(crate) async fn received(&mut self, amount: Amount) -> Result<(), FungibleTokenError> {
        self.my_balance.set(self.my_balance.get().try_add(amount)?);
        Ok(())
    }
}
