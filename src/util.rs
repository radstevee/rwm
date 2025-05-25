/// Creates a zeroed out u8 array of size [`N`].
pub const fn zeroed<const N: usize>() -> [u8; N] {
    let mut arr = [0; N];
    let mut idx = 0;
    while idx < N {
        arr[idx] = 0;
        idx += 1;
    }
    arr
}
