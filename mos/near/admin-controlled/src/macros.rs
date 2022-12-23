#[macro_export]
macro_rules! impl_admin_controlled {
    ($contract: ident, $paused: ident) => {
        use admin_controlled::{AdminControlled as AdminControlledInner, Mask as MaskInner};

        #[near_bindgen]
        impl AdminControlledInner for $contract {
            fn get_paused(&self) -> MaskInner {
                self.$paused
            }

            fn set_paused(&mut self, paused: MaskInner) {
                assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
                self.$paused = paused;
            }
        }
    };
}
