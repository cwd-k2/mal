#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::driver) enum OptimizationProfile {
    Baseline,
    Production,
}

impl OptimizationProfile {
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
    fn keeps_toolchain_optimization_out_of_the_semantic_profile() {
        assert_eq!(
            OptimizationProfile::Baseline.arguments(),
            ["-O0", "-Wno-error=#warnings"]
        );
        assert_eq!(
            OptimizationProfile::Production.arguments(),
            ["-O2", "-flto"]
        );
    }
}
