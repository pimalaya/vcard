/// Decode a parsed value into its real model type, resolving its leaves and
/// escapes and borrowing the leaf text where no unescaping is needed.
pub trait VcardDecode {
    /// The model type this parsed value decodes into, borrowing the leaves.
    type Output<'a>
    where
        Self: 'a;

    /// Decode into the model type, borrowing from `self`.
    fn decode(&self) -> Self::Output<'_>;
}
