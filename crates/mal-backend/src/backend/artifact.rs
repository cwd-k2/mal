pub struct LlvmArtifacts {
    pub module: String,
    pub shim: String,
    pub header: String,
    pub runtime: Vec<RuntimeSource>,
}

pub struct RuntimeSource {
    pub name: &'static str,
    pub contents: &'static str,
}
