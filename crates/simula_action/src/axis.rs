use bevy::{prelude::*, reflect::FromReflect, utils::HashMap};
use std::hash::Hash;

/// Stores the position data of the input devices of type `T`.
///
/// The values are stored as `f32`s, which range from [`ActionAxis::MIN`] to [`ActionAxis::MAX`], inclusive.
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ActionAxis<T: Eq + Hash + Clone + Send + Sync + 'static + FromReflect> {
    /// The name of the axis.
    name: String,
    /// The position data of the input devices.
    axis: HashMap<T, f32>,
}

impl<T> Default for ActionAxis<T>
where
    T: Eq + Hash + Clone + Send + Sync + 'static + FromReflect,
{
    fn default() -> Self {
        ActionAxis {
            name: std::any::type_name::<T>().to_string(),
            axis: HashMap::default(),
        }
    }
}

impl<T> ActionAxis<T>
where
    T: Eq + Hash + Clone + Send + Sync + 'static + FromReflect,
{
    /// The smallest possible axis value.
    pub const MIN: f32 = -1.0;

    /// The largest possible axis value.
    pub const MAX: f32 = 1.0;

    /// The name of the axis.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the position data of the `input_device` to `position_data`.
    ///
    /// The `position_data` is clamped to be between [`ActionAxis::MIN`] and [`ActionAxis::MAX`], inclusive.
    ///
    /// If the `input_device`:
    /// - was present before, the position data is updated, and the old value is returned.
    /// - wasn't present before, [None] is returned.
    pub fn set(&mut self, input_device: T, position_data: f32) -> Option<f32> {
        let new_position_data = position_data.clamp(Self::MIN, Self::MAX);
        self.axis.insert(input_device, new_position_data)
    }

    /// Returns a position data corresponding to the `input_device`.
    pub fn get(&self, input_device: T) -> Option<f32> {
        self.axis.get(&input_device).copied()
    }

    /// Removes the position data of the `input_device`, returning the position data if the input device was previously set.
    pub fn remove(&mut self, input_device: T) -> Option<f32> {
        self.axis.remove(&input_device)
    }

    /// Remove all position data.
    pub fn clear(&mut self) {
        self.axis.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::gamepad::{Gamepad, GamepadButton, GamepadButtonType};

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect)]
    pub enum TestGamepad {
        Gamepad(GamepadButton),
    }

    #[test]
    fn test_axis_set() {
        let cases = [
            (-1.5, Some(-1.0)),
            (-1.1, Some(-1.0)),
            (-1.0, Some(-1.0)),
            (-0.9, Some(-0.9)),
            (-0.1, Some(-0.1)),
            (0.0, Some(0.0)),
            (0.1, Some(0.1)),
            (0.9, Some(0.9)),
            (1.0, Some(1.0)),
            (1.1, Some(1.0)),
            (1.6, Some(1.0)),
        ];

        for (value, expected) in cases {
            let gamepad_button = TestGamepad::Gamepad(GamepadButton::new(
                Gamepad::new(1),
                GamepadButtonType::RightTrigger,
            ));
            let mut axis = ActionAxis::<TestGamepad>::default();

            axis.set(gamepad_button, value);

            let actual = axis.get(gamepad_button);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_axis_remove() {
        let cases = [-1.0, -0.9, -0.1, 0.0, 0.1, 0.9, 1.0];

        for value in cases {
            let gamepad_button = TestGamepad::Gamepad(GamepadButton::new(
                Gamepad::new(1),
                GamepadButtonType::RightTrigger,
            ));
            let mut axis = ActionAxis::<TestGamepad>::default();

            axis.set(gamepad_button, value);
            assert!(axis.get(gamepad_button).is_some());

            axis.remove(gamepad_button);
            let actual = axis.get(gamepad_button);
            let expected = None;

            assert_eq!(expected, actual);
        }
    }
}
