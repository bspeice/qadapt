/// Anything that can be initialized with a `const` value.
pub(crate) trait ConstInit {
    /// The `const` default initializer value for `Self`.
    const INIT: Self;
}