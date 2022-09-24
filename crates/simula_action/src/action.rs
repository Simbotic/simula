use bevy::prelude::*;
use bevy::utils::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Action<T: Eq + Hash + Clone + Send + Sync + 'static> {
    /// The name of the action.
    name: String,
    /// A collection of every action that is currently on.
    on: HashSet<T>,
    /// A collection of every action that has just been entered.
    on_enter: HashSet<T>,
    /// A collection of every action that has just been exited.
    on_exit: HashSet<T>,
}

impl<T: Eq + Hash + Clone + Send + Sync + 'static> Default for Action<T> {
    fn default() -> Self {
        Self {
            name: std::any::type_name::<T>().to_string(),
            on: Default::default(),
            on_enter: Default::default(),
            on_exit: Default::default(),
        }
    }
}

impl<T> Action<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    /// Returns the name of the action.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Registers an on_enter for the given `action`.
    pub fn enter(&mut self, action: T) {
        // Returns `true` if the `action` wasn't on.
        if self.on.insert(action) {
            self.on_enter.insert(action);
        }
    }

    /// Returns `true` if the `action` has been on.
    pub fn on(&self, action: T) -> bool {
        self.on.contains(&action)
    }

    /// Returns `true` if any `action` in `actions` has been on.
    pub fn any_on(&self, actions: impl IntoIterator<Item = T>) -> bool {
        actions.into_iter().any(|it| self.on(it))
    }

    /// Registers an on_exit for the given `action`.
    pub fn exit(&mut self, action: T) {
        // Returns `true` if the `action` was on.
        if self.on.remove(&action) {
            self.on_exit.insert(action);
        }
    }

    /// Registers a exit for all currently on actions.
    pub fn exit_all(&mut self) {
        // Move all actions from on into on_exit
        self.on_exit.extend(self.on.drain());
    }

    /// Returns `true` if the `action` has just been entered.
    pub fn on_enter(&self, action: T) -> bool {
        self.on_enter.contains(&action)
    }

    /// Returns `true` if any `action` in `actions` has just been entered.
    pub fn any_on_enter(&self, actions: impl IntoIterator<Item = T>) -> bool {
        actions.into_iter().any(|it| self.on_enter(it))
    }

    /// Clears the `on_enter` state of the `action` and returns `true` if the `action` has just been entered.
    ///
    /// Future calls to [`Action::on_enter`] for the given action will return false until a new on_enter event occurs.
    pub fn clear_on_enter(&mut self, action: T) -> bool {
        self.on_enter.remove(&action)
    }

    /// Returns `true` if the `action` has just been exited.
    pub fn on_exit(&self, action: T) -> bool {
        self.on_exit.contains(&action)
    }

    /// Returns `true` if any item in `actions` has just been exited.
    pub fn any_on_exit(&self, actions: impl IntoIterator<Item = T>) -> bool {
        actions.into_iter().any(|it| self.on_exit(it))
    }

    /// Clears the `on_exit` state of the `action` and returns `true` if the `action` has just been exited.
    ///
    /// Future calls to [`Action::on_exit`] for the given action will return false until a new on_exit event occurs.
    pub fn clear_on_exit(&mut self, action: T) -> bool {
        self.on_exit.remove(&action)
    }

    /// Clears the `pressed`, `on_enter` and `on_exit` data of the `action`.
    pub fn reset(&mut self, action: T) {
        self.on.remove(&action);
        self.on_enter.remove(&action);
        self.on_exit.remove(&action);
    }

    /// Clears the `on`, `on_enter`, and `on_exit` data for every action.
    ///
    /// See also [`Action::clear`] for simulating elapsed time steps.
    pub fn reset_all(&mut self) {
        self.on.clear();
        self.on_enter.clear();
        self.on_exit.clear();
    }

    /// Clears the `on_enter` and `on_exit` data for every action.
    ///
    /// See also [`Action::reset_all`] for a full reset.
    pub fn clear(&mut self) {
        self.on_enter.clear();
        self.on_exit.clear();
    }

    /// An iterator visiting every on action in arbitrary order.
    pub fn get_on(&self) -> impl ExactSizeIterator<Item = &T> {
        self.on.iter()
    }

    /// An iterator visiting every on_enter action in arbitrary order.
    pub fn get_on_enter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.on_enter.iter()
    }

    /// An iterator visiting every on_exit action in arbitrary order.
    pub fn get_on_exit(&self) -> impl ExactSizeIterator<Item = &T> {
        self.on_exit.iter()
    }
}

#[cfg(test)]
mod test {
    use super::Action;

    /// Used for testing the functionality of [`Action`].
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    enum DummyAction {
        Action1,
        Action2,
    }

    #[test]
    fn test_enter() {
        let mut action = Action::default();
        assert!(!action.on.contains(&DummyAction::Action1));
        assert!(!action.on_enter.contains(&DummyAction::Action1));
        action.enter(DummyAction::Action1);
        assert!(action.on_enter.contains(&DummyAction::Action1));
        assert!(action.on.contains(&DummyAction::Action1));
    }

    #[test]
    fn test_on() {
        let mut action = Action::default();
        assert!(!action.on(DummyAction::Action1));
        action.enter(DummyAction::Action1);
        assert!(action.on(DummyAction::Action1));
    }

    #[test]
    fn test_any_on() {
        let mut action = Action::default();
        assert!(!action.any_on([DummyAction::Action1]));
        assert!(!action.any_on([DummyAction::Action2]));
        assert!(!action.any_on([DummyAction::Action1, DummyAction::Action2]));
        action.enter(DummyAction::Action1);
        assert!(action.any_on([DummyAction::Action1]));
        assert!(!action.any_on([DummyAction::Action2]));
        assert!(action.any_on([DummyAction::Action1, DummyAction::Action2]));
    }

    #[test]
    fn test_exit() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        assert!(action.on.contains(&DummyAction::Action1));
        assert!(!action.on_exit.contains(&DummyAction::Action1));
        action.exit(DummyAction::Action1);
        assert!(!action.on.contains(&DummyAction::Action1));
        assert!(action.on_exit.contains(&DummyAction::Action1));
    }

    #[test]
    fn test_exit_all() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);
        action.exit_all();
        assert!(action.on.is_empty());
        assert!(action.on_exit.contains(&DummyAction::Action1));
        assert!(action.on_exit.contains(&DummyAction::Action2));
    }

    #[test]
    fn test_on_enter() {
        let mut action = Action::default();
        assert!(!action.on_enter(DummyAction::Action1));
        action.enter(DummyAction::Action1);
        assert!(action.on_enter(DummyAction::Action1));
    }

    #[test]
    fn test_any_on_enter() {
        let mut action = Action::default();
        assert!(!action.any_on_enter([DummyAction::Action1]));
        assert!(!action.any_on_enter([DummyAction::Action2]));
        assert!(!action.any_on_enter([DummyAction::Action1, DummyAction::Action2]));
        action.enter(DummyAction::Action1);
        assert!(action.any_on_enter([DummyAction::Action1]));
        assert!(!action.any_on_enter([DummyAction::Action2]));
        assert!(action.any_on_enter([DummyAction::Action1, DummyAction::Action2]));
    }

    #[test]
    fn test_clear_on_enter() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        assert!(action.on_enter(DummyAction::Action1));
        action.clear_on_enter(DummyAction::Action1);
        assert!(!action.on_enter(DummyAction::Action1));
    }

    #[test]
    fn test_on_exit() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        assert!(!action.on_exit(DummyAction::Action1));
        action.exit(DummyAction::Action1);
        assert!(action.on_exit(DummyAction::Action1));
    }

    #[test]
    fn test_any_on_exit() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        assert!(!action.any_on_exit([DummyAction::Action1]));
        assert!(!action.any_on_exit([DummyAction::Action2]));
        assert!(!action.any_on_exit([DummyAction::Action1, DummyAction::Action2]));
        action.exit(DummyAction::Action1);
        assert!(action.any_on_exit([DummyAction::Action1]));
        assert!(!action.any_on_exit([DummyAction::Action2]));
        assert!(action.any_on_exit([DummyAction::Action1, DummyAction::Action2]));
    }

    #[test]
    fn test_clear_on_exit() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        action.exit(DummyAction::Action1);
        assert!(action.on_exit(DummyAction::Action1));
        action.clear_on_exit(DummyAction::Action1);
        assert!(!action.on_exit(DummyAction::Action1));
    }

    #[test]
    fn test_reset() {
        let mut action = Action::default();

        // Pressed
        action.enter(DummyAction::Action1);
        assert!(action.on(DummyAction::Action1));
        assert!(action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));
        action.reset(DummyAction::Action1);
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));

        // Released
        action.enter(DummyAction::Action1);
        action.exit(DummyAction::Action1);
        assert!(!action.on(DummyAction::Action1));
        assert!(action.on_enter(DummyAction::Action1));
        assert!(action.on_exit(DummyAction::Action1));
        action.reset(DummyAction::Action1);
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));
    }

    #[test]
    fn test_reset_all() {
        let mut action = Action::default();

        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);
        action.exit(DummyAction::Action2);
        assert!(action.on.contains(&DummyAction::Action1));
        assert!(action.on_enter.contains(&DummyAction::Action1));
        assert!(action.on_exit.contains(&DummyAction::Action2));
        action.reset_all();
        assert!(action.on.is_empty());
        assert!(action.on_enter.is_empty());
        assert!(action.on_exit.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut action = Action::default();

        // Pressed
        action.enter(DummyAction::Action1);
        assert!(action.on(DummyAction::Action1));
        assert!(action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));
        action.clear();
        assert!(action.on(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));

        // Released
        action.enter(DummyAction::Action1);
        action.exit(DummyAction::Action1);
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(action.on_exit(DummyAction::Action1));
        action.clear();
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action1));
    }

    #[test]
    fn test_get_on() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);
        let all_on = action.get_on();
        assert_eq!(all_on.len(), 2);
        for on_action in all_on {
            assert!(action.on.contains(on_action));
        }
    }

    #[test]
    fn test_get_on_enter() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);
        let on_enter = action.get_on_enter();
        assert_eq!(on_enter.len(), 2);
        for on_enter_action in on_enter {
            assert!(action.on_enter.contains(on_enter_action));
        }
    }

    #[test]
    fn test_get_on_exit() {
        let mut action = Action::default();
        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);
        action.exit(DummyAction::Action1);
        action.exit(DummyAction::Action2);
        let on_exit = action.get_on_exit();
        assert_eq!(on_exit.len(), 2);
        for on_exit_action in on_exit {
            assert!(action.on_exit.contains(on_exit_action));
        }
    }

    #[test]
    fn test_general_action_handling() {
        let mut action = Action::default();

        // Test entering
        action.enter(DummyAction::Action1);
        action.enter(DummyAction::Action2);

        // Check if they were `on_enter` (entered on this update)
        assert!(action.on_enter(DummyAction::Action1));
        assert!(action.on_enter(DummyAction::Action2));

        // Check if they are also marked as `on`
        assert!(action.on(DummyAction::Action1));
        assert!(action.on(DummyAction::Action2));

        // Clear the `action`, removing `on_enter` and `on_exit`
        action.clear();

        // Check if they're marked `on_enter`
        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on_enter(DummyAction::Action2));

        // Check if they're marked as `on`
        assert!(action.on(DummyAction::Action1));
        assert!(action.on(DummyAction::Action2));

        // Release the actions and check state
        action.exit(DummyAction::Action1);
        action.exit(DummyAction::Action2);

        // Check if they're marked as `on_exit` (exited on this update)
        assert!(action.on_exit(DummyAction::Action1));
        assert!(action.on_exit(DummyAction::Action2));

        // Check that they're not incorrectly marked as `on`
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on(DummyAction::Action2));

        // Clear the `Action` and check for removal from `on_exit`
        action.clear();

        // Check that they're not incorrectly marked as `on_exit`
        assert!(!action.on_exit(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action2));

        // Set up an `Action` to test resetting
        let mut action = Action::default();

        action.enter(DummyAction::Action1);
        action.exit(DummyAction::Action2);

        // Reset the `Action` and test if it was reset correctly
        action.reset(DummyAction::Action1);
        action.reset(DummyAction::Action2);

        assert!(!action.on_enter(DummyAction::Action1));
        assert!(!action.on(DummyAction::Action1));
        assert!(!action.on_exit(DummyAction::Action2));
    }
}
