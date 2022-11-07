use bevy::prelude::*;
use bevy_egui::egui::Ui;
use bevy_inspector_egui::{Context, Inspectable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Reflect, Default, Deref, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(pub i128);

impl From<i128> for Amount {
    fn from(amount: i128) -> Self {
        Self(amount)
    }
}

impl From<u32> for Amount {
    fn from(amount: u32) -> Self {
        Self(amount as i128)
    }
}

impl From<i32> for Amount {
    fn from(amount: i32) -> Self {
        Self(amount as i128)
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i128(self.0)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        i128::deserialize(deserializer).map(Self)
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

/// An asset for a specific class
#[derive(
    Default, Component, Reflect, Deref, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Asset<const CLASS_ID: u64, const ASSET_ID: u64>(pub Amount);

impl<const CLASS_ID: u64, const ASSET_ID: u64> Inspectable for Asset<CLASS_ID, ASSET_ID> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut Ui, _options: Self::Attributes, _context: &mut Context) -> bool {
        let changed;
        ui.label(format!("Asset<{}, {}>", CLASS_ID, ASSET_ID));
        let mut amount_text = self.0 .0.to_string();
        changed = ui.text_edit_singleline(&mut amount_text).changed();
        if changed {
            self.0 .0 = amount_text.parse().unwrap();
        }
        changed
    }
}

impl<const CLASS_ID: u64, const ASSET_ID: u64> Asset<CLASS_ID, ASSET_ID> {
    pub fn new(balance: Amount) -> Self {
        Self(balance)
    }

    pub fn class_id() -> u64 {
        CLASS_ID
    }

    pub fn asset_id() -> u64 {
        ASSET_ID
    }
}

impl<const CLASS_ID: u64, const ASSET_ID: u64> Serialize for Asset<CLASS_ID, ASSET_ID> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i128(self.0 .0)
    }
}

impl<'de, const CLASS_ID: u64, const ASSET_ID: u64> Deserialize<'de> for Asset<CLASS_ID, ASSET_ID> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        i128::deserialize(deserializer).map(Amount).map(Self)
    }
}

impl<const CLASS_ID: u64, const ASSET_ID: u64> From<i128> for Asset<CLASS_ID, ASSET_ID> {
    fn from(amount: i128) -> Self {
        Self(amount.into())
    }
}

#[derive(Component)]
pub struct AssetBalance {
    pub class_id: u64,
    pub asset_id: u64,
    pub balance: Amount,
}

impl<const CLASS_ID: u64, const ASSET_ID: u64> From<Asset<CLASS_ID, ASSET_ID>> for AssetBalance {
    fn from(asset: Asset<CLASS_ID, ASSET_ID>) -> Self {
        Self {
            class_id: CLASS_ID,
            asset_id: ASSET_ID,
            balance: asset.0,
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

    impl From<AgentToken> for AssetBalance {
        fn from(token: AgentToken) -> Self {
            match token {
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

    impl From<MissionToken> for AssetBalance {
        fn from(token: MissionToken) -> Self {
            match token {
                MissionToken::Time(asset) => asset.into(),
                MissionToken::Reward(asset) => asset.into(),
            }
        }
    }

    fn generate_agent_tokens() -> Vec<AgentToken> {
        vec![
            AgentToken::Labor(20.into()),
            AgentToken::Energy(10000.into()),
            AgentToken::Waste(0.into()),
        ]
    }

    fn generate_mission_tokens() -> Vec<MissionToken> {
        vec![
            MissionToken::Time(100.into()),
            MissionToken::Reward(1000.into()),
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

        let balance = all_tokens
            .into_iter()
            .fold(Amount(0), |acc, i| acc + i.balance);

        assert_eq!(balance, Amount(11120));
    }

    #[test]
    fn assets_serialize_works() {
        let tokens = generate_agent_tokens();
        println!("tokens: {:?}", tokens);
        let ser_ron = ron::ser::to_string(&tokens).unwrap();
        println!("RON: {}", ser_ron);
        let des_tokens: Vec<AgentToken> = ron::de::from_str(&ser_ron).unwrap();
        assert_eq!(tokens, des_tokens);
    }

    #[test]
    fn assets_deserialize_works() {
        let tokens = generate_agent_tokens();
        let ser_ron = "[Labor(20),Energy(10000),Waste(0)]";
        let des_tokens: Vec<AgentToken> = ron::de::from_str(&ser_ron).unwrap();
        assert_eq!(tokens, des_tokens);
    }
}
