use bevy::prelude::*;

pub mod account;
pub mod asset;
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
