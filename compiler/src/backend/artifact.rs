pub(crate) struct LlvmArtifacts {
    pub(crate) module: String,
    pub(crate) shim: String,
    pub(crate) header: String,
    pub(crate) runtime: Vec<RuntimeSource>,
}

pub(crate) struct RuntimeSource {
    pub(crate) name: &'static str,
    pub(crate) contents: &'static str,
}
