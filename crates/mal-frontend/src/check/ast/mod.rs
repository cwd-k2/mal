//! Typed program representation emitted only after language-rule admission.

use crate::resolve::ast::{
    ExternalOperationId, LambdaId, TypeBinding, TypeId, ValueBinding, ValueId, ValueReference,
};
use mal_syntax::ast::Node;
use mal_syntax::source::FileId;
use mal_syntax::source::Span;
use std::{collections::HashSet, sync::Arc};

mod expression;
mod primitive;
mod program;
mod type_terms;

pub use expression::*;
pub use primitive::*;
pub use program::*;
pub use type_terms::*;
