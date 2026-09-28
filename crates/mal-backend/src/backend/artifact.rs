/// Complete generated input set returned to the filesystem-owning driver.
pub struct LlvmArtifacts {
    /// Program-specific LLVM module.
    pub module: String,
    /// C11 process and host bridge for the module.
    pub shim: String,
    /// Public C host interface for the program.
    pub header: String,
    /// Program-independent C11 sources selected from referenced runtime symbols.
    pub runtime: Vec<RuntimeSource>,
}

/// One checked-in runtime source selected for a generated program.
pub struct RuntimeSource {
    /// Stable artifact filename.
    pub name: &'static str,
    /// Source contents compiled by the driver.
    pub contents: &'static str,
}
