use std::ffi::OsStr;
use std::process::Command;

use super::Error;

mod optimization;

pub(super) use optimization::OptimizationMode;

pub(super) const CLANG: &str = "clang";

pub(super) struct Target {
    pub(super) triple: String,
    pub(super) data_layout: String,
}

pub(super) fn host_target() -> Result<Target, Error> {
    let output = Command::new(CLANG)
        .args(["-S", "-emit-llvm", "-x", "c", "/dev/null", "-o", "-"])
        .output()
        .map_err(|error| Error::tool_start(OsStr::new(CLANG), error))?;
    if !output.status.success() {
        return Err(Error::tool_failure(
            OsStr::new(CLANG),
            output.status.code(),
            &output.stderr,
        ));
    }
    let module = String::from_utf8_lossy(&output.stdout);
    Ok(Target {
        triple: quoted_module_property(&module, "target triple")?,
        data_layout: quoted_module_property(&module, "target datalayout")?,
    })
}

fn quoted_module_property(module: &str, property: &str) -> Result<String, Error> {
    let prefix = format!("{property} = \"");
    let value = module
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.strip_suffix('"'))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::new(format!("malc: Clang did not report {property}")))?;
    Ok(value.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_target_properties_from_clang_ir() {
        let module = "target datalayout = \"e-p:64:64\"\ntarget triple = \"x86_64-test\"\n";
        assert_eq!(
            quoted_module_property(module, "target datalayout").unwrap(),
            "e-p:64:64"
        );
        assert_eq!(
            quoted_module_property(module, "target triple").unwrap(),
            "x86_64-test"
        );
    }
}
