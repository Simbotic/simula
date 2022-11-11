use crate::account::{Account, AccountId};
use crate::prelude::{AssetInfo, Amount};
use crate::wallet::{Wallet, WalletId};
use bevy::prelude::*;
use core::fmt::Debug;

pub struct AssetBuilder<T> where T: AssetInfo + Debug + Clone{
    asset: T, 
    attributes: T::AssetAttributes
}

impl<T> AssetBuilder<T>
where
    T: AssetInfo + Clone + Debug,
{
    pub fn amount(&mut self, asset: T, attributes: T::AssetAttributes) -> &mut Self {
        self.asset = asset;
        self.attributes = attributes;
        self
    }

    pub fn build(&self, cmd: &mut ChildBuilder) -> T {
        let name = format!("{:?}", self.asset)
            .chars()
            .take_while(|&ch| ch != '(')
            .collect::<String>();
        cmd.spawn()
            .insert(self.asset.clone())
            .insert(self.attributes.clone())
            .insert(Name::new(format!("Asset: {}", name)));
        self.asset.clone()
    }
}

#[derive(Component, Clone, Debug)]
pub struct DefaultToken{
    _value: i128,
}

#[derive(Clone,Component,Default)]
pub struct DefaultAssetAtributtes{
}

impl AssetInfo for DefaultToken{
    type AssetAttributes = DefaultAssetAtributtes;

    fn name(&self) -> &'static str {
        todo!()
    }

    fn amount(&self) -> Amount {
        todo!()
    }

    fn is_draggable(&self) -> bool {
        todo!()
    }

    fn class_id(&self)->u64 {
        todo!()
    }

    fn asset_id(&self)->u64 {
        todo!()
    }

    fn drag(&mut self)-> bool {
        todo!()
    }

    fn drop(&mut self, src_class_id: u64, src_asset_id: u64, source_amount: Amount)-> bool {
        todo!()
    }

    fn push_as_children(&self,commands: &mut Commands, parent: Entity) {
        todo!()
    }
}

impl Default for DefaultToken{
    fn default() -> Self {
        Self { _value: Default::default() }
    }
}

trait NewTrait: AssetInfo + Debug + Clone{}

impl Default for AssetBuilder<DefaultToken>{
    fn default() -> Self {
        Self { asset: Default::default(), attributes: Default::default() }
    }
}

#[derive(Default)]
pub struct AccountBuilder<T> where T:AssetInfo + Debug + Clone {
    pub id: AccountId,
    pub assets: Vec<AssetBuilder<T>>,
}

impl<T> AccountBuilder<T>
where
    T: AssetInfo + Default + Component + Clone + Debug,
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
pub struct WalletBuilder<T> where T: AssetInfo + Debug + Clone{
    pub id: WalletId,
    pub accounts: Vec<AccountBuilder<T>>,
} 

impl<T> WalletBuilder<T>
where
    T: AssetInfo + Default + Component + Clone + Debug,
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
