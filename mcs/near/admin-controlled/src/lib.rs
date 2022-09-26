pub mod macros;
pub use macros::*;

pub type Mask = u128;

pub trait AdminControlled {
    /// Return the current mask representing all paused events.
    fn get_paused(&self) -> Mask;

    /// Update mask with all paused events.
    /// Implementor is responsible for guaranteeing that this function can only be
    /// called by owner of the contract.
    fn set_paused(&mut self, paused: Mask);

    /// Return if the contract is paused for the current flag
    fn is_paused(&self, flag: Mask) -> bool {
        (self.get_paused() & flag) != 0
    }

    fn check_not_paused(&self, flag: Mask) {
        assert!(!self.is_paused(flag));
    }
}
