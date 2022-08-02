#[macro_export]
macro_rules! impl_admin_controlled {
    ($contract: ident, $paused: ident) => {
        use admin_controlled::{AdminControlled as AdminControlledInner, Mask as MaskInner};
        use near_sdk as near_sdk_inner;

        #[near_bindgen]
        impl AdminControlledInner for $contract {
            fn get_paused(&self) -> MaskInner {
                self.$paused
            }

            fn set_paused(&mut self, paused: MaskInner) {
                near_sdk_inner::assert_self();
                self.$paused = paused;
            }
        }
    };
}
