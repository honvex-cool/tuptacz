// Macro for definitions of id-like types that use the same underlying represenation, but we want to distinguish them in code.
#[macro_export]
macro_rules! id_type {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub $type);
    };
}
