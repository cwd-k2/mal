macro_rules! llvm_scalar_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ($literal:literal) => { $literal };
}

macro_rules! llvm_type_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (void) => { $crate::backend::llvm::syntax::Type::Void };
    (ptr) => { $crate::backend::llvm::syntax::Type::Pointer };
    (float) => { $crate::backend::llvm::syntax::Type::Float };
    (double) => { $crate::backend::llvm::syntax::Type::Double };
    (int($bits:tt)) => {
        $crate::backend::llvm::syntax::Type::integer(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($bits),
        )
    };
    (array($length:tt, $($element:tt)+)) => {
        $crate::backend::llvm::syntax::Type::array(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($length),
            $crate::backend::llvm::syntax::llvm_type_normalized!($($element)+),
        )
    };
    (structure([$($field:tt)*])) => {
        $crate::backend::llvm::syntax::Type::structure(
            $crate::backend::llvm::syntax::llvm_types_normalized!($($field)*)
        )
    };
}

macro_rules! llvm_types_normalized {
    ($($field:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut fields = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_types_items_normalized!(fields; $($field)*);
        fields
    }};
}

macro_rules! llvm_types_items_normalized {
    ($fields:ident;) => {};
    ($fields:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $fields.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_types_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_types_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::llvm::syntax::llvm_type_normalized!($kind($($argument)*)));
        $crate::backend::llvm::syntax::llvm_types_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; $primitive:ident $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::llvm::syntax::llvm_type_normalized!($primitive));
        $crate::backend::llvm::syntax::llvm_types_items_normalized!($fields; $($($rest)*)?);
    };
}

macro_rules! llvm_parameter_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (#[immarg] $($parameter:tt)+) => {{
        $crate::backend::llvm::syntax::llvm_parameter_normalized!($($parameter)+)
            .with_attribute($crate::backend::llvm::syntax::ParameterAttribute::ImmArg)
    }};
    (_ : $kind:ident($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type_normalized!($kind($($ty)*)),
        )
    };
    (_ : $primitive:ident) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type_normalized!($primitive),
        )
    };
    (_ : (@rust $($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::unnamed({ $($ty)* })
    };
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type_normalized!($kind($($ty)*)),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
        )
    };
    ($name:tt : $primitive:ident) => {
        $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type_normalized!($primitive),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
        )
    };
    ($name:tt : (@rust $($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::named(
            { $($ty)* },
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
        )
    };
}

macro_rules! llvm_parameters_normalized {
    ($($parameter:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut parameters = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!(parameters; $($parameter)*);
        parameters
    }};
}

macro_rules! llvm_parameters_items_normalized {
    ($parameters:ident;) => {};
    ($parameters:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!(#[immarg] $name : $kind($($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : $primitive:ident $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!(#[immarg] $name : $primitive));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : (@rust $($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!(#[immarg] $name : (@rust $($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!($name : $kind($($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $primitive:ident $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!($name : $primitive));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : (@rust $($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter_normalized!($name : (@rust $($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
}

macro_rules! llvm_function_attributes_normalized {
    ($($attribute:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut attributes = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!(attributes; $($attribute)*);
        attributes
    }};
}

macro_rules! llvm_function_attributes_items_normalized {
    ($attributes:ident;) => {};
    ($attributes:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $attributes.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $attributes.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; nofree $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoFree);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; noinline $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoInline);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; nounwind $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoUnwind);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; willreturn $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::WillReturn);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; memory_none $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::MemoryNone);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; memory_argmem_read $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::MemoryArgMemRead);
        $crate::backend::llvm::syntax::llvm_function_attributes_items_normalized!($attributes; $($($rest)*)?);
    };
}

macro_rules! llvm_signature_result_normalized {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $kind:ident($($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_signature_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; $linkage;
            $crate::backend::llvm::syntax::llvm_type_normalized!($kind($($result)*))
        }
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $primitive:ident) => {
        $crate::backend::llvm::syntax::llvm_signature_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; $linkage;
            $crate::backend::llvm::syntax::llvm_type_normalized!($primitive)
        }
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; (@rust $($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_signature_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; $linkage; { $($result)* }
        }
    };
}

macro_rules! llvm_signature_build_normalized {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $result:expr) => {{
        let signature = $crate::backend::llvm::syntax::FunctionSignature::new(
            $result,
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
            $crate::backend::llvm::syntax::llvm_parameters_normalized!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes_normalized!($($attribute)*)
        );
        match $linkage {
            Some(linkage) => signature.with_linkage(linkage),
            None => signature,
        }
    }};
}

macro_rules! llvm_signature_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (#[linkage(internal)] #[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result_normalized! {
            $name; [$($parameter)*]; [$($attribute)*];
            Some($crate::backend::llvm::syntax::Linkage::Internal); $($result)+
        }
    };
    (#[linkage(internal)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result_normalized! {
            $name; [$($parameter)*]; [];
            Some($crate::backend::llvm::syntax::Linkage::Internal); $($result)+
        }
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; None; $($result)+
        }
    };
    (fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result_normalized! {
            $name; [$($parameter)*]; []; None; $($result)+
        }
    };
}

macro_rules! llvm_type {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_type_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_parameter {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_parameter_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_parameters {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_parameters_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_function_attributes {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_function_attributes_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_signature {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_signature_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    llvm_function_attributes_items_normalized, llvm_function_attributes_normalized,
    llvm_parameter_normalized, llvm_parameters_items_normalized, llvm_parameters_normalized,
    llvm_scalar_normalized, llvm_signature_build_normalized, llvm_signature_normalized,
    llvm_signature_result_normalized, llvm_type_normalized, llvm_types_items_normalized,
    llvm_types_normalized,
};

pub(in crate::backend) use {
    llvm_function_attributes, llvm_parameter, llvm_parameters, llvm_signature, llvm_type,
};
