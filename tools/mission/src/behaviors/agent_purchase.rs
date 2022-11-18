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

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct AgentPurchase {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    start: f64,
}

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
    time: Res<Time>,
    mut agents_work: Query<
        (
            Entity,
            &mut AgentPurchase,
            &mut BehaviorRunning,
            &mut BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
    machines: Query<(Entity, &Children, &MachineType<T>), (With<Machine>, Without<Agent>)>,
    mut agents: Query<(Entity, &Children, &AgentPurchaseType<T>), (With<Agent>, Without<Machine>)>,
    mut wallets: Query<(&Wallet, &Children), Without<Machine>>,
    mut accounts: Query<(&Account, &Children)>,
    mut assets: Query<(Entity, &mut T)>,
) {
    for (agent_work_entity, mut work, mut running, node) in agents_work.iter_mut() {
        if let Some(tree_entity) = node.tree {
            if let Ok((_agent, agent_children, agent_purchase_type)) = agents.get_mut(tree_entity) {
                let mut source_asset = None;
                let mut target_asset = None;

                for (_machine, machine_children, machine_type) in &machines {
                    if agent_purchase_type.0.name() == machine_type.0.name() {
                        for machine_child in machine_children {
                            if let Ok((_machine_wallet, machine_wallet_accounts)) =
                                wallets.get(machine_child.to_owned())
                            {
                                for machine_wallet_account in machine_wallet_accounts {
                                    if let Ok((_machine_account, machine_account_assets)) =
                                        accounts.get(machine_wallet_account.to_owned())
                                    {
                                        for machine_account_asset in machine_account_assets {
                                            if let Ok((machine_asset_entity, machine_asset)) =
                                                assets.get(machine_account_asset.to_owned())
                                            {
                                                if agent_purchase_type.0.name()
                                                    == machine_asset.name()
                                                {
                                                    source_asset = Some(machine_asset_entity);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                for agent_child in agent_children {
                    if let Ok((_agent_wallet, agent_wallet_accounts)) =
                        wallets.get_mut(*agent_child)
                    {
                        for agent_wallet_account in agent_wallet_accounts {
                            if let Ok((_agent_account, agent_account_assets)) =
                                accounts.get_mut(*agent_wallet_account)
                            {
                                for agent_account_asset in agent_account_assets {
                                    if let Ok((agent_asset_entity, agent_asset)) =
                                        assets.get(*agent_account_asset)
                                    {
                                        if agent_purchase_type.0.name() == agent_asset.name() {
                                            target_asset = Some(agent_asset_entity);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                trade_assets(source_asset.as_mut(), target_asset.as_mut(), &mut assets);
            }
        }
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            work.start = time.seconds_since_startup();
        }
        let duration = time.seconds_since_startup() - work.start;
        if duration > work.duration {
            commands.entity(agent_work_entity).insert(BehaviorSuccess);
        }
    }
}

fn trade_assets<T: AssetInfo>(
    source: Option<&mut Entity>,
    target: Option<&mut Entity>,
    assets: &mut Query<(Entity, &mut T)>,
) {
    if let (Some(source_asset), Some(target_asset)) = (source, target) {
        let mut asset_class_id = None;
        let mut asset_asset_id = None;
        let mut asset_amount: Option<Amount> = None;

        if let Ok((_, mut source_asset)) = assets.get_mut(*source_asset) {
            asset_class_id = Some(source_asset.class_id());
            asset_asset_id = Some(source_asset.asset_id());
            asset_amount = Some(source_asset.amount());

            if let (Some(asset_class_id), Some(asset_asset_id), Some(asset_amount)) =
                (asset_class_id, asset_asset_id, asset_amount)
            {
                source_asset.drop(asset_class_id, asset_asset_id, Amount(-asset_amount.0));
            }
        }

        if let Ok((_, mut target_asset)) = assets.get_mut(*target_asset) {
            if let (Some(asset_class_id), Some(asset_asset_id), Some(asset_amount)) =
                (asset_class_id, asset_asset_id, asset_amount)
            {
                target_asset.drop(asset_class_id, asset_asset_id, Amount(asset_amount.0));
            }
        }
    }
}
