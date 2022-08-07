/// Returns a reference to the elements of `$slice` as an array, verifying that
/// the slice is of length `$len`.
///
/// source: https://github.com/rust-lang/rfcs/issues/1833
#[macro_export]
macro_rules! slice_as_array_ref {
    ($slice:expr, $len:expr) => {
        {
            fn slice_as_array_ref<T>(slice: &[T]) -> Result<&[T; $len], ()> {
                if slice.len() != $len {
                    return Err(());
                }
                Ok(unsafe {
                    &*(slice.as_ptr() as *const [T; $len])
                })
            }
            slice_as_array_ref($slice)
        }
    }
}