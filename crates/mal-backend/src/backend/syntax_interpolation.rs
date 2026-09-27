macro_rules! normalize_syntax_interpolation {
    ([$($callback:tt)+]; $($input:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [root $($callback)+] [] $($input)*
        )
    };

    (@tokens [$($continuation:tt)*] [$($output:tt)*]) => {
        $crate::backend::normalize_syntax_interpolation!(
            @continue [$($continuation)*] [$($output)*]
        )
    };
    (@tokens [$($continuation:tt)*] [$($output:tt)*]
        #{ $($rust:tt)* } $($rest:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [$($continuation)*] [$($output)* (@rust $($rust)*)] $($rest)*
        )
    };
    (@tokens [$($continuation:tt)*] [$($output:tt)*]
        ($($inner:tt)*) $($rest:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [parentheses [$($continuation)*] [$($output)*] [$($rest)*]] [] $($inner)*
        )
    };
    (@tokens [$($continuation:tt)*] [$($output:tt)*]
        [$($inner:tt)*] $($rest:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [brackets [$($continuation)*] [$($output)*] [$($rest)*]] [] $($inner)*
        )
    };
    (@tokens [$($continuation:tt)*] [$($output:tt)*]
        {$($inner:tt)*} $($rest:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [braces [$($continuation)*] [$($output)*] [$($rest)*]] [] $($inner)*
        )
    };
    (@tokens [$($continuation:tt)*] [$($output:tt)*] $token:tt $($rest:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [$($continuation)*] [$($output)* $token] $($rest)*
        )
    };

    (@continue [root $($callback:tt)+] [$($output:tt)*]) => {
        $($callback)+!($($output)*)
    };
    (@continue [parentheses [$($continuation:tt)*] [$($output:tt)*] [$($rest:tt)*]]
        [$($inner:tt)*]) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [$($continuation)*] [$($output)* ($($inner)*)] $($rest)*
        )
    };
    (@continue [brackets [$($continuation:tt)*] [$($output:tt)*] [$($rest:tt)*]]
        [$($inner:tt)*]) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [$($continuation)*] [$($output)* [$($inner)*]] $($rest)*
        )
    };
    (@continue [braces [$($continuation:tt)*] [$($output:tt)*] [$($rest:tt)*]]
        [$($inner:tt)*]) => {
        $crate::backend::normalize_syntax_interpolation!(
            @tokens [$($continuation)*] [$($output)* {$($inner)*}] $($rest)*
        )
    };
}

pub(in crate::backend) use normalize_syntax_interpolation;
