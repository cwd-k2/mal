macro_rules! llvm_scalar {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ($literal:literal) => { $literal };
}

macro_rules! llvm_type {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (void) => { $crate::backend::llvm::syntax::Type::Void };
    (ptr) => { $crate::backend::llvm::syntax::Type::Pointer };
    (float) => { $crate::backend::llvm::syntax::Type::Float };
    (double) => { $crate::backend::llvm::syntax::Type::Double };
    (int($bits:tt)) => {
        $crate::backend::llvm::syntax::Type::integer(
            $crate::backend::llvm::syntax::llvm_scalar!($bits),
        )
    };
    (array($length:tt, $($element:tt)+)) => {
        $crate::backend::llvm::syntax::Type::array(
            $crate::backend::llvm::syntax::llvm_scalar!($length),
            $crate::backend::llvm::syntax::llvm_type!($($element)+),
        )
    };
    (structure([$($field:tt)*])) => {
        $crate::backend::llvm::syntax::Type::structure(
            $crate::backend::llvm::syntax::llvm_types!($($field)*)
        )
    };
}

macro_rules! llvm_types {
    ($($field:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut fields = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_types_items!(fields; $($field)*);
        fields
    }};
}

macro_rules! llvm_types_items {
    ($fields:ident;) => {};
    ($fields:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_types_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_types_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::llvm::syntax::llvm_type!($kind($($argument)*)));
        $crate::backend::llvm::syntax::llvm_types_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; $primitive:ident $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::llvm::syntax::llvm_type!($primitive));
        $crate::backend::llvm::syntax::llvm_types_items!($fields; $($($rest)*)?);
    };
}

macro_rules! llvm_parameter {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (#[immarg] $($parameter:tt)+) => {{
        $crate::backend::llvm::syntax::llvm_parameter!($($parameter)+)
            .with_attribute($crate::backend::llvm::syntax::ParameterAttribute::ImmArg)
    }};
    (_ : $kind:ident($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($kind($($ty)*)),
        )
    };
    (_ : $primitive:ident) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($primitive),
        )
    };
    (_ : {{ $($ty:tt)* }}) => {
        $crate::backend::llvm::syntax::Parameter::unnamed({ $($ty)* })
    };
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type!($kind($($ty)*)),
            $crate::backend::llvm::syntax::llvm_scalar!($name),
        )
    };
    ($name:tt : $primitive:ident) => {
        $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type!($primitive),
            $crate::backend::llvm::syntax::llvm_scalar!($name),
        )
    };
    ($name:tt : {{ $($ty:tt)* }}) => {
        $crate::backend::llvm::syntax::Parameter::named(
            { $($ty)* },
            $crate::backend::llvm::syntax::llvm_scalar!($name),
        )
    };
}

macro_rules! llvm_parameters {
    ($($parameter:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut parameters = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_parameters_items!(parameters; $($parameter)*);
        parameters
    }};
}

macro_rules! llvm_parameters_items {
    ($parameters:ident;) => {};
    ($parameters:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(#[immarg] $name : $kind($($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : $primitive:ident $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(#[immarg] $name : $primitive));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[immarg] $name:tt : {{ $($ty:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(#[immarg] $name : {{ $($ty)* }}));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $kind($($ty)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $primitive:ident $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $primitive));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : {{ $($ty:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : {{ $($ty)* }}));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
}

macro_rules! llvm_function_attributes {
    ($($attribute:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut attributes = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!(attributes; $($attribute)*);
        attributes
    }};
}

macro_rules! llvm_function_attributes_items {
    ($attributes:ident;) => {};
    ($attributes:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $attributes.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $attributes.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; nofree $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoFree);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; noinline $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoInline);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; nounwind $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::NoUnwind);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; willreturn $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::WillReturn);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; memory_none $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::MemoryNone);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
    ($attributes:ident; memory_argmem_read $(, $($rest:tt)*)?) => {
        $attributes.push($crate::backend::llvm::syntax::FunctionAttribute::MemoryArgMemRead);
        $crate::backend::llvm::syntax::llvm_function_attributes_items!($attributes; $($($rest)*)?);
    };
}

macro_rules! llvm_signature_result {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $kind:ident($($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name; [$($parameter)*]; [$($attribute)*]; $linkage;
            $crate::backend::llvm::syntax::llvm_type!($kind($($result)*))
        )
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $primitive:ident) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name; [$($parameter)*]; [$($attribute)*]; $linkage;
            $crate::backend::llvm::syntax::llvm_type!($primitive)
        )
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; {{ $($result:tt)* }}) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name; [$($parameter)*]; [$($attribute)*]; $linkage; { $($result)* }
        )
    };
}

macro_rules! llvm_signature_build {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $linkage:expr; $result:expr) => {{
        let signature = $crate::backend::llvm::syntax::FunctionSignature::new(
            $result,
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            $crate::backend::llvm::syntax::llvm_parameters!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes!($($attribute)*)
        );
        match $linkage {
            Some(linkage) => signature.with_linkage(linkage),
            None => signature,
        }
    }};
}

macro_rules! llvm_signature {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (#[linkage(internal)] #[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result!(
            $name; [$($parameter)*]; [$($attribute)*];
            Some($crate::backend::llvm::syntax::Linkage::Internal); $($result)+
        )
    };
    (#[linkage(internal)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result!(
            $name; [$($parameter)*]; [];
            Some($crate::backend::llvm::syntax::Linkage::Internal); $($result)+
        )
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result!(
            $name; [$($parameter)*]; [$($attribute)*]; None; $($result)+
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::llvm::syntax::llvm_signature_result!(
            $name; [$($parameter)*]; []; None; $($result)+
        )
    };
}

pub(in crate::backend) use {
    llvm_function_attributes, llvm_function_attributes_items, llvm_parameter, llvm_parameters,
    llvm_parameters_items, llvm_scalar, llvm_signature, llvm_signature_build,
    llvm_signature_result, llvm_type, llvm_types, llvm_types_items,
};
