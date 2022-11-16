use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_mission::{
    account::Account,
    asset::Amount,
    asset_info::AssetInfo,
    machine::{Machine, MachineType},
    wallet::Wallet,
};

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct MachineProduction {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    start: f64,
}

impl BehaviorInfo for MachineProduction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Machine production";
    const DESC: &'static str = "Produce something";
}

#[derive(Debug, Default, Component, Reflect, Clone)]
pub struct ProductionTimer(Timer);

pub struct MachineProductionNodePlugin<T: AssetInfo>(pub T);

impl<T: AssetInfo> Plugin for MachineProductionNodePlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProductionTimer(Timer::from_seconds(1.0, true)))
            .add_system(run::<T>);
    }
}

pub fn run<T: AssetInfo>(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ProductionTimer>,
    mut machine_production: Query<
        (
            Entity,
            &mut MachineProduction,
            &mut BehaviorRunning,
            &mut BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
    mut machines: Query<(Entity, &Children, &MachineType<T>), With<Machine>>,
    mut wallets: Query<(&mut Wallet, &Children), Without<Machine>>,
    mut accounts: Query<(&mut Account, &Children)>,
    mut assets: Query<&mut T>,
) {
    for (machine_production_entity, mut production, mut running, node) in
        machine_production.iter_mut()
    {
        if let Some(tree_entity) = node.tree {
            if !running.on_enter_handled {
                running.on_enter_handled = true;
                production.start = time.seconds_since_startup();
            }
            let duration = time.seconds_since_startup() - production.start;
            if duration > production.duration {
                commands
                    .entity(machine_production_entity)
                    .insert(BehaviorSuccess);
            }
            if timer.0.tick(time.delta()).just_finished() {
                if let Ok((_machine, machine_children, machine_type)) =
                    machines.get_mut(tree_entity)
                {
                    for machine_child in machine_children {
                        if let Ok((_wallet, wallet_accounts)) = wallets.get_mut(*machine_child) {
                            for wallet_account in wallet_accounts {
                                if let Ok((_account, account_assets)) =
                                    accounts.get_mut(*wallet_account)
                                {
                                    for account_asset in account_assets {
                                        if let Ok(mut asset) = assets.get_mut(*account_asset) {
                                            if machine_type.0.name() == asset.name() {
                                                let asset_class_id = asset.class_id();
                                                let asset_asset_id = asset.asset_id();
                                                asset.drop(asset_class_id, asset_asset_id, Amount(1));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
