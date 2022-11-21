use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_mission::{
    account::Account,
    agent::{Agent, AgentPurchaseType},
    asset::Amount,
    asset_info::AssetInfo,
    machine::{Machine, MachineType},
    wallet::Wallet,
};
use std::iter::zip;

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct AgentPurchase {}

impl BehaviorInfo for AgentPurchase {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent purchase";
    const DESC: &'static str = "Purchase items";
}

pub struct AgentPurchaseNodePlugin<T: AssetInfo>(pub T);

impl<T: AssetInfo> Plugin for AgentPurchaseNodePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system(run::<T>);
    }
}

pub fn run<T: AssetInfo>(
    mut commands: Commands,
    mut agents_work: Query<(Entity, &mut AgentPurchase, &mut BehaviorNode), BehaviorRunQuery>,
    machines: Query<(Entity, &Children, &MachineType<T>), (With<Machine>, Without<Agent>)>,
    mut agents: Query<(Entity, &Children, &AgentPurchaseType<T>), (With<Agent>, Without<Machine>)>,
    wallets: Query<(&Wallet, &Children), Without<Machine>>,
    mut accounts: Query<(&Account, &Children)>,
    mut assets: Query<(Entity, &mut T)>,
) {
    for (agent_work_entity, _purchase, node) in agents_work.iter_mut() {
        if let Some(tree_entity) = node.tree {
            if let Ok((_agent, agent_children, agent_purchase_type)) = agents.get_mut(tree_entity) {
                let mut source_accounts = None;
                let mut target_accounts = None;

                for (_machine, machine_children, machine_type) in &machines {
                    if agent_purchase_type.0.class_id() == machine_type.0.class_id()
                        && agent_purchase_type.0.asset_id() == machine_type.0.asset_id()
                    {
                        for machine_child in machine_children {
                            if let Ok((_machine_wallet, machine_wallet_accounts)) =
                                wallets.get(machine_child.to_owned())
                            {
                                source_accounts = Some(machine_wallet_accounts);
                            }
                        }
                    }
                }

                for agent_child in agent_children {
                    if let Ok((_agent_wallet, agent_wallet_accounts)) =
                        wallets.get(agent_child.to_owned())
                    {
                        target_accounts = Some(agent_wallet_accounts);
                    }
                }
                if source_accounts.is_some() && target_accounts.is_some() {
                    iterate_accounts(
                        source_accounts.unwrap(),
                        target_accounts.unwrap(),
                        &mut accounts,
                        &mut assets,
                        &agent_purchase_type.0,
                    );
                }
            }
        }
        commands.entity(agent_work_entity).insert(BehaviorSuccess);
    }
}

fn iterate_accounts<T: AssetInfo>(
    source_accounts: &Children,
    target_accounts: &Children,
    accounts: &mut Query<(&Account, &Children)>,
    mut assets: &mut Query<(Entity, &mut T)>,
    asset_type: &T,
) {
    let mut iter = zip(source_accounts, target_accounts);

    loop {
        if let Some((iter_source_account, iter_target_account)) = iter.next() {
            let mut source_assets = None;
            let mut target_assets = None;

            if let Ok((_account, account_assets)) = accounts.get(iter_source_account.to_owned()) {
                source_assets = Some(account_assets);
            }
            if let Ok((_account, account_assets)) = accounts.get(iter_target_account.to_owned()) {
                target_assets = Some(account_assets);
            }

            if source_assets.is_some() && target_assets.is_some() {
                iterate_assets(
                    source_assets.unwrap(),
                    target_assets.unwrap(),
                    &mut assets,
                    &asset_type,
                );
            }
        } else {
            break;
        }
    }
}

fn iterate_assets<T: AssetInfo>(
    source_assets: &Children,
    target_assets: &Children,
    assets: &mut Query<(Entity, &mut T)>,
    asset_type: &T,
) {
    let mut source_asset = None;
    let mut target_asset = None;
    for source_asset_entity in source_assets {
        if let Ok((entity, asset)) = assets.get(*source_asset_entity) {
            if asset.class_id() == asset_type.class_id()
                && asset.asset_id() == asset_type.asset_id()
            {
                source_asset = Some(entity);
                break;
            }
        }
    }
    for target_asset_entity in target_assets {
        if let Ok((entity, asset)) = assets.get(*target_asset_entity) {
            if asset.class_id() == asset_type.class_id()
                && asset.asset_id() == asset_type.asset_id()
            {
                target_asset = Some(entity);
                break;
            }
        }
    }
    if source_asset.is_some() && target_asset.is_some() {
        trade_assets(source_asset.unwrap(), target_asset.unwrap(), assets);
    }
}

fn trade_assets<T: AssetInfo>(
    source: Entity,
    target: Entity,
    assets: &mut Query<(Entity, &mut T)>,
) {
    let mut asset_class_id = None;
    let mut asset_asset_id = None;
    let mut asset_amount: Option<Amount> = None;

    if let Ok((_, mut source)) = assets.get_mut(source) {
        asset_class_id = Some(source.class_id());
        asset_asset_id = Some(source.asset_id());
        asset_amount = Some(source.amount());

        if let (Some(asset_class_id), Some(asset_asset_id), Some(asset_amount)) =
            (asset_class_id, asset_asset_id, asset_amount)
        {
            source.drop(asset_class_id, asset_asset_id, Amount(-asset_amount.0));
        }
    }

    if let Ok((_, mut target)) = assets.get_mut(target) {
        if let (Some(asset_class_id), Some(asset_asset_id), Some(asset_amount)) =
            (asset_class_id, asset_asset_id, asset_amount)
        {
            target.drop(asset_class_id, asset_asset_id, asset_amount);
        }
    }
}
