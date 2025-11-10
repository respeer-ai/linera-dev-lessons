use linera_sdk::{
    ensure,
    linera_base_types::{AccountOwner, Amount},
    views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext},
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
    pub balances: MapView<AccountOwner, Amount>,
    pub balance: RegisterView<Amount>,
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
        owner: AccountOwner,
        amount: Amount,
    ) -> Result<Amount, FungibleTokenError> {
        let target_amount = try_amount_mul(amount, *self.mint_ratio.get())?;
        let total_balance = *self.balance.get();

        log::info!("Mint amount {} target_amount {}", amount, target_amount);

        if target_amount.gt(&total_balance) {
            return Err(FungibleTokenError::InsufficientFunds);
        }

        let balance = self.balances.get(&owner).await?.unwrap_or(Amount::ZERO);
        self.balance.set(total_balance.try_sub(target_amount)?);
        self.balances
            .insert(&owner, balance.try_add(target_amount)?)?;

        Ok(target_amount)
    }

    pub(crate) async fn transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
    ) -> Result<(), FungibleTokenError> {
        let from_balance = self.balances.get(&from).await?.unwrap_or(Amount::ZERO);
        let to_balance = self.balances.get(&to).await?.unwrap_or(Amount::ZERO);

        ensure!(
            from_balance.gt(&amount),
            FungibleTokenError::InsufficientFunds
        );

        self.balances.insert(&from, from_balance.try_sub(amount)?)?;
        self.balances.insert(&to, to_balance.try_add(amount)?)?;

        Ok(())
    }
}
