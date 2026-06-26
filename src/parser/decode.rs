/// Decode a parsed value into its real model type, resolving its leaves and
/// escapes and borrowing the source where no unescaping is needed.
pub trait VcardDecode<'a> {
    /// The model type this parsed value decodes into.
    type Output;

    /// Decode against the source `input`.
    fn decode(&'a self, input: &'a str) -> Self::Output;
}
