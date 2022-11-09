use crate::prelude::Wallet;
use crate::Account;

fn trim_id(id: String) -> String {
    id.get(0..8).unwrap_or_default().to_string()
}

pub fn trim_wallet(wallet: &Wallet) -> String {
    trim_id(wallet.wallet_id.to_string())
}

pub fn trim_account(account: &Account) -> String {
    trim_id(account.account_id.to_string())
}