use bevy::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// pub struct AccountBuilder

/// Match SugarFunge account size
#[derive(Default, Reflect, PartialEq, Clone)]
pub struct AccountId {
    #[reflect(ignore)]
    raw_id: [u8; 32],
    pub id: String,
}

#[derive(Default, Debug, Reflect, Component, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct Account {
    pub account_id: AccountId,
}

impl ToString for AccountId {
    fn to_string(&self) -> String {
        hex::encode(self)
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.raw_id[..]
    }
}

impl TryFrom<&[u8]> for AccountId {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() == 32 {
            let mut inner = [0u8; 32];
            inner.copy_from_slice(data);
            Ok(AccountId {
                raw_id: inner,
                id: hex::encode(inner),
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for AccountId {
    type Error = ();

    fn try_from(hex_str: String) -> Result<Self, Self::Error> {
        if hex_str.len() == 64 {
            if let Ok(hex_dec) = hex::decode(hex_str) {
                let mut inner = [0u8; 32];
                inner.copy_from_slice(&hex_dec);
                Ok(AccountId {
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

impl std::fmt::Debug for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl Serialize for AccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self))
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let account_id_hex = hex::decode(String::deserialize(deserializer)?)
            .map_err(|e| de::Error::custom(format!("{:?}", e)))?;
        AccountId::try_from(account_id_hex.as_ref())
            .map_err(|e| de::Error::custom(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id() {
        let hex_str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
        let hex_dec = hex::decode(hex_str).unwrap();

        let account_id = AccountId::try_from(hex_dec.as_ref()).unwrap();
        let account_ser = ron::to_string(&account_id).unwrap();

        println!("AccountId: {:?}", account_id);
        println!("RON: {}", account_ser);

        assert_eq!(account_id.to_string(), hex_str);
        assert_eq!(
            account_id,
            AccountId::try_from(hex_str.to_string()).unwrap()
        );
        assert_eq!(account_id, AccountId::try_from(hex_dec.as_ref()).unwrap());
        assert_eq!(account_ser, format!("\"{}\"", hex_str));
    }

    #[test]
    fn account_serialize_works() {
        let account = Account {
            account_id: AccountId::try_from(
                "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60".to_string(),
            )
            .unwrap(),
        };
        let account_ser = ron::to_string(&account).unwrap();
        let account_des: Account = ron::from_str(&account_ser).unwrap();
        assert_eq!(account, account_des);
        println!("RON: {}", account_ser);
    }
}
