macro_rules! llvm_scalar {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    ($literal:literal) => { $literal };
}

macro_rules! llvm_type {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
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
    ($fields:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_types_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; { $($rust:tt)* } $(, $($rest:tt)*)?) => {
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

macro_rules! llvm_parameter_attributes {
    ($parameter:ident;) => {};
    ($parameter:ident; immarg $(, $rest:ident)*) => {
        $parameter = $parameter.with_attribute(
            $crate::backend::llvm::syntax::ParameterAttribute::ImmArg,
        );
        $crate::backend::llvm::syntax::llvm_parameter_attributes!($parameter; $($rest),*);
    };
}

macro_rules! llvm_parameter {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (_ : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($kind($($type)*)),
        );
        $crate::backend::llvm::syntax::llvm_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    (_ : $primitive:ident [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($primitive),
        );
        $crate::backend::llvm::syntax::llvm_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    (_ : { $($type:tt)* } [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::llvm::syntax::Parameter::unnamed({ $($type)* });
        $crate::backend::llvm::syntax::llvm_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    (_ : $kind:ident($($type:tt)*)) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($kind($($type)*)),
        )
    };
    (_ : $primitive:ident) => {
        $crate::backend::llvm::syntax::Parameter::unnamed(
            $crate::backend::llvm::syntax::llvm_type!($primitive),
        )
    };
    (_ : { $($type:tt)* }) => {
        $crate::backend::llvm::syntax::Parameter::unnamed({ $($type)* })
    };
    ($name:tt : $($type:tt)+ [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type!($($type)+),
            $crate::backend::llvm::syntax::llvm_scalar!($name),
        );
        $crate::backend::llvm::syntax::llvm_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    ($name:tt : $($type:tt)+) => {
        $crate::backend::llvm::syntax::Parameter::named(
            $crate::backend::llvm::syntax::llvm_type!($($type)+),
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
    ($parameters:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; { $($rust:tt)* } $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?], $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : $kind($($type)*) [$($attribute),*]
        ));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $primitive:ident [$($attribute:ident),* $(,)?], $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : $primitive [$($attribute),*]
        ));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } [$($attribute:ident),* $(,)?], $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : { $($type)* } [$($attribute),*]
        ));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*), $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $kind($($type)*)));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $primitive:ident, $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $primitive));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : { $($type:tt)* }, $($rest:tt)*) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : { $($type)* }));
        $crate::backend::llvm::syntax::llvm_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?] $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : $kind($($type)*) [$($attribute),*]
        ));
    };
    ($parameters:ident; $name:tt : $primitive:ident [$($attribute:ident),* $(,)?] $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : $primitive [$($attribute),*]
        ));
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } [$($attribute:ident),* $(,)?] $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!(
            $name : { $($type)* } [$($attribute),*]
        ));
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $kind($($type)*)));
    };
    ($parameters:ident; $name:tt : $primitive:ident $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : $primitive));
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } $(,)?) => {
        $parameters.push($crate::backend::llvm::syntax::llvm_parameter!($name : { $($type)* }));
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
    ($attributes:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $attributes.extend({ $($rust)* });
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

macro_rules! llvm_signature_build {
    ($name:tt; [$($parameter:tt)*]; result { $($result:tt)* }; [$($attribute:tt)*]; linkage { $($linkage:tt)* }) => {{
        let signature = $crate::backend::llvm::syntax::FunctionSignature::new(
            { $($result)* },
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            $crate::backend::llvm::syntax::llvm_parameters!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes!($($attribute)*)
        );
        match { $($linkage)* } {
            Some(linkage) => signature.with_linkage(linkage),
            None => signature,
        }
    }};
}

macro_rules! llvm_signature {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (internal fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*); attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($kind($($result)*)) };
            [$($attribute)*];
            linkage { Some($crate::backend::llvm::syntax::Linkage::Internal) }
        )
    };
    (internal fn $name:tt($($parameter:tt)*) -> $primitive:ident; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($primitive) };
            [$($attribute)*];
            linkage { Some($crate::backend::llvm::syntax::Linkage::Internal) }
        )
    };
    (internal fn $name:tt($($parameter:tt)*) -> { $($result:tt)* }; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name; [$($parameter)*]; result { $($result)* }; [$($attribute)*];
            linkage { Some($crate::backend::llvm::syntax::Linkage::Internal) }
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*); attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($kind($($result)*)) };
            [$($attribute)*];
            linkage { None }
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $primitive:ident; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($primitive) };
            [$($attribute)*];
            linkage { None }
        )
    };
    (fn $name:tt($($parameter:tt)*) -> { $($result:tt)* }; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_signature_build!(
            $name; [$($parameter)*]; result { $($result)* }; [$($attribute)*]; linkage { None }
        )
    };
}

pub(in crate::backend) use {
    llvm_function_attributes, llvm_function_attributes_items, llvm_parameter,
    llvm_parameter_attributes, llvm_parameters, llvm_parameters_items, llvm_scalar, llvm_signature,
    llvm_signature_build, llvm_type, llvm_types, llvm_types_items,
};
