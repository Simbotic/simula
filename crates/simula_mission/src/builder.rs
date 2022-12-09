use crate::account::{Account, AccountId};
use crate::wallet::{Wallet, WalletId};
use bevy::prelude::*;
use core::fmt::Debug;

#[derive(Default)]
pub struct AssetBuilder<T> {
    asset: T,
}

impl<T> AssetBuilder<T>
where
    T: Component + Clone + Debug,
{
    pub fn amount(&mut self, asset: T) -> &mut Self {
        self.asset = asset;
        self
    }

    pub fn build(&self, cmd: &mut ChildBuilder) -> T {
        let name = format!("{:?}", self.asset)
            .chars()
            .take_while(|&ch| ch != '(')
            .collect::<String>();
        cmd.spawn_empty()
            .insert(self.asset.clone())
            .insert(Name::new(format!("Asset: {}", name)));
        self.asset.clone()
    }
}

#[derive(Default)]
pub struct AccountBuilder<T> {
    pub id: AccountId,
    pub assets: Vec<AssetBuilder<T>>,
}

impl<T> AccountBuilder<T>
where
    T: Default + Component + Clone + Debug,
{
    pub fn id(&mut self, id: &str) -> &mut Self {
        self.id = AccountId::try_from(id.to_string()).unwrap_or_default();
        self
    }

    pub fn with_asset(&mut self, f: impl FnOnce(&mut AssetBuilder<T>)) -> &mut Self {
        let mut asset = AssetBuilder::<T>::default();
        f(&mut asset);
        self.assets.push(asset);
        self
    }

    pub fn build(&self, cmd: &mut ChildBuilder) -> Account {
        let name = self
            .id
            .to_string()
            .get(0..8)
            .unwrap_or_default()
            .to_string();
        cmd.spawn_empty()
            .insert(Account {
                account_id: self.id.clone(),
            })
            .with_children(|account| {
                for asset in &self.assets {
                    asset.build(account);
                }
            })
            .insert(Name::new(format!("Account: {}", name)));

        Account {
            account_id: self.id.clone(),
        }
    }
}

#[derive(Default)]
pub struct WalletBuilder<T> {
    pub id: WalletId,
    pub accounts: Vec<AccountBuilder<T>>,
}

impl<T> WalletBuilder<T>
where
    T: Default + Component + Clone + Debug,
{
    pub fn id(&mut self, id: &str) -> &mut Self {
        self.id = WalletId::try_from(id.to_string()).unwrap_or_default();
        self
    }

    pub fn with_account(&mut self, f: impl FnOnce(&mut AccountBuilder<T>)) -> &mut Self {
        let mut account = AccountBuilder::<T>::default();
        f(&mut account);
        self.accounts.push(account);
        self
    }

    pub fn build(&self, cmd: &mut Commands) -> Entity {
        let name = self
            .id
            .to_string()
            .get(0..8)
            .unwrap_or_default()
            .to_string();
        cmd.spawn_empty()
            .insert(Wallet {
                wallet_id: self.id.clone(),
            })
            .with_children(|wallet| {
                for account in &self.accounts {
                    account.build(wallet);
                }
            })
            .insert(Name::new(format!("Wallet: {}", name)))
            .id()
    }
}
