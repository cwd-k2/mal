#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::driver) enum OptimizationMode {
    Baseline,
    Production,
}

impl OptimizationMode {
    pub(in crate::driver) const fn arguments(self) -> &'static [&'static str] {
        match self {
            Self::Baseline => &["-O0", "-Wno-error=#warnings"],
            Self::Production => &["-O2", "-flto"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_toolchain_optimization_out_of_the_semantic_mode() {
        assert_eq!(
            OptimizationMode::Baseline.arguments(),
            ["-O0", "-Wno-error=#warnings"]
        );
        assert_eq!(OptimizationMode::Production.arguments(), ["-O2", "-flto"]);
    }
}
