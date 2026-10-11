/// Stacking and conflict resolution policy for layered configuration sources.
///
/// Dictates how incoming configuration keys interact with already existing keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PushPolicy {
    /// **Prefer Incoming / Override** (Figment `merge`):
    ///
    /// When an incoming key conflicts with an existing one, the incoming value
    /// overwrites the existing value. Dictionaries/maps are recursively united.
    #[default]
    Merge,

    /// **Prefer Existing / Fallback** (Figment `join`):
    ///
    /// When an incoming key conflicts with an existing one, the existing value
    /// is preserved and the incoming value is ignored.
    /// Dictionaries/maps are still recursively united, but existing leaves win.
    Join,
}
