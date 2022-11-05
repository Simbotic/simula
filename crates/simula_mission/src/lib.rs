use account::{Account, AccountId};
use bevy::prelude::*;
use core::fmt::Debug;
use wallet::{Wallet, WalletId};

pub mod account;
pub mod agent;
pub mod asset;
pub mod environment;
pub mod game;
pub mod wallet;

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<account::Account>()
            .register_type::<account::AccountId>()
            .register_type::<wallet::Wallet>()
            .register_type::<wallet::WalletId>();
    }
}

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
        cmd.spawn()
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
        cmd.spawn()
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
        cmd.spawn()
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

#[cfg(test)]
mod tests {
    use ta::indicators::ExponentialMovingAverage;
    use ta::Next;

    #[test]
    fn it_works() {
        // it can return an error, when an invalid length is passed (e.g. 0)
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        assert_eq!(ema.next(2.0), 2.0);
        assert_eq!(ema.next(5.0), 3.5);
        assert_eq!(ema.next(1.0), 2.25);
        assert_eq!(ema.next(6.25), 4.25);
    }
}
