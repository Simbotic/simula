use bevy::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// Match SugarFunge wallet size
#[derive(Default, Reflect, PartialEq, Clone)]
pub struct WalletId {
    #[reflect(ignore)]
    raw_id: [u8; 32],
    pub id: String,
}

#[derive(Default, Debug, Reflect, Component, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct Wallet {
    pub wallet_id: WalletId,
}

impl ToString for WalletId {
    fn to_string(&self) -> String {
        hex::encode(self)
    }
}

impl AsRef<[u8]> for WalletId {
    fn as_ref(&self) -> &[u8] {
        &self.raw_id[..]
    }
}

impl TryFrom<&[u8]> for WalletId {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() == 32 {
            let mut inner = [0u8; 32];
            inner.copy_from_slice(data);
            Ok(WalletId {
                raw_id: inner,
                id: hex::encode(inner),
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for WalletId {
    type Error = ();

    fn try_from(hex_str: String) -> Result<Self, Self::Error> {
        if hex_str.len() == 64 {
            if let Ok(hex_dec) = hex::decode(hex_str) {
                let mut inner = [0u8; 32];
                inner.copy_from_slice(&hex_dec);
                Ok(WalletId {
                    raw_id: inner,
                    id: hex::encode(inner),
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for WalletId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl Serialize for WalletId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self))
    }
}

impl<'de> Deserialize<'de> for WalletId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wallet_id_hex = hex::decode(String::deserialize(deserializer)?)
            .map_err(|e| de::Error::custom(format!("{:?}", e)))?;
        WalletId::try_from(wallet_id_hex.as_ref())
            .map_err(|e| de::Error::custom(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_id() {
        let hex_str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
        let hex_dec = hex::decode(hex_str).unwrap();

        let wallet_id = WalletId::try_from(hex_dec.as_ref()).unwrap();
        let wallet_ser = ron::to_string(&wallet_id).unwrap();

        println!("WalletId: {:?}", wallet_id);
        println!("RON: {}", wallet_ser);

        assert_eq!(wallet_id.to_string(), hex_str);
        assert_eq!(wallet_id, WalletId::try_from(hex_str.to_string()).unwrap());
        assert_eq!(wallet_id, WalletId::try_from(hex_dec.as_ref()).unwrap());
        assert_eq!(wallet_ser, format!("\"{}\"", hex_str));
    }

    #[test]
    fn wallet_serialize_works() {
        let wallet = Wallet {
            wallet_id: WalletId::try_from(
                "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60".to_string(),
            )
            .unwrap(),
        };
        let wallet_ser = ron::to_string(&wallet).unwrap();
        let wallet_des: Wallet = ron::from_str(&wallet_ser).unwrap();
        assert_eq!(wallet, wallet_des);
        println!("RON: {}", wallet_ser);
    }
}
