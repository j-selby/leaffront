pub trait VersionInfo {
    /// Returns version information about this implementation
    fn version() -> String;
}
