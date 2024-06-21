/// The type of relation between nodes or paths in the transport network.
pub enum NetworkRelationType {
    /// The nodes or paths can be connected.
    Connectable,

    /// The nodes or paths should not be connected.
    ToAvoid,
}
