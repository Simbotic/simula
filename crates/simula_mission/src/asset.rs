use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Balance of an asset
pub type Balance = u128;

/// Signed version of Balance
pub type Amount = i128;

/// An asset for a specific class
#[derive(
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
    Deref,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[reflect(Component)]
pub struct Asset<const CLASS_ID: u64, const ASSET_ID: u64> {
    pub balance: Amount,
}

/// Implement Into<Asset> for Amount
impl<const CLASS_ID: u64, const ASSET_ID: u64> Into<Asset<CLASS_ID, ASSET_ID>> for Amount {
    fn into(self) -> Asset<CLASS_ID, ASSET_ID> {
        Asset { balance: self }
    }
}

pub struct AssetBalance {
    pub class_id: u64,
    pub asset_id: u64,
    pub balance: Amount,
}

impl<const CLASS_ID: u64, const ASSET_ID: u64> Into<AssetBalance> for Asset<CLASS_ID, ASSET_ID> {
    fn into(self) -> AssetBalance {
        AssetBalance {
            class_id: CLASS_ID,
            asset_id: ASSET_ID,
            balance: self.balance,
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum AgentToken {
        Labor(Asset<1000, 0>),
        Energy(Asset<1000, 1>),
        Waste(Asset<1000, 2>),
    }

    impl Into<AssetBalance> for AgentToken {
        fn into(self) -> AssetBalance {
            match self {
                AgentToken::Labor(asset) => asset.into(),
                AgentToken::Energy(asset) => asset.into(),
                AgentToken::Waste(asset) => asset.into(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum MissionToken {
        Time(Asset<2000, 0>),
        Reward(Asset<2000, 1>),
    }

    impl Into<AssetBalance> for MissionToken {
        fn into(self) -> AssetBalance {
            match self {
                MissionToken::Time(asset) => asset.into(),
                MissionToken::Reward(asset) => asset.into(),
            }
        }
    }

    fn generate_agent_tokens() -> Vec<AgentToken> {
        vec![
            AgentToken::Labor(Asset { balance: 20 }),
            AgentToken::Energy(Asset { balance: 10000 }),
            AgentToken::Waste(Asset { balance: 0 }),
        ]
    }

    fn generate_mission_tokens() -> Vec<MissionToken> {
        vec![
            MissionToken::Time(Asset { balance: 100 }),
            MissionToken::Reward(Asset { balance: 1000 }),
        ]
    }

    #[test]
    fn assets_lists_works() {
        let agent_tokens: Vec<AssetBalance> = generate_agent_tokens()
            .into_iter()
            .map(|i| i.into())
            .collect();

        let mission_tokens: Vec<AssetBalance> = generate_mission_tokens()
            .into_iter()
            .map(|i| i.into())
            .collect();

        let all_tokens: Vec<AssetBalance> = agent_tokens
            .into_iter()
            .chain(mission_tokens.into_iter())
            .collect();

        let balance = all_tokens.into_iter().fold(0, |acc, i| acc + i.balance);

        assert_eq!(balance, 11120);
    }

    #[test]
    fn assets_serialize_works() {
        let tokens = generate_agent_tokens();
        let ser_ron = ron::ser::to_string(&tokens).unwrap();
        println!("RON: {}", ser_ron);
        let des_tokens: Vec<AgentToken> = ron::de::from_str(&ser_ron).unwrap();
        assert_eq!(tokens, des_tokens);
    }

    #[test]
    fn assets_deserialize_works() {
        let tokens = generate_agent_tokens();
        let ser_ron = "[Labor((balance:20)),Energy((balance:10000)),Waste((balance:0))]";
        let des_tokens: Vec<AgentToken> = ron::de::from_str(&ser_ron).unwrap();
        assert_eq!(tokens, des_tokens);
    }
}
