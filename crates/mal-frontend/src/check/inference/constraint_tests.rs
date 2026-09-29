use std::collections::{HashMap, HashSet};

use mal_syntax::source::{FileId, Span};

use super::constraint::resolve_substitution;
use crate::check::ast::{Kind, Type};
use crate::resolve::ast::TypeId;

#[test]
fn reports_normalization_limit_while_resolving_inference_substitutions() {
    let span = Span::new(FileId::new(7), 11, 19);
    let constructor = TypeId(100);
    let resolved = TypeId(101);
    let bound = Type::Bound {
        index: 0,
        kind: Kind::Type,
    };
    let huge = Type::Abstraction {
        parameter_kind: Kind::Type,
        body: Type::Product(vec![bound; 65_536].into()).into(),
    };
    let application = Type::Application {
        constructor: Type::Parameter {
            id: constructor,
            name: "F".into(),
            kind: Kind::function(Kind::Type, Kind::Type),
        }
        .into(),
        argument: Type::Unit.into(),
        kind: Kind::Type,
        span,
    };
    let substitutions = HashMap::from([(constructor, huge), (resolved, application)]);

    let error = resolve_substitution(resolved, &substitutions, &mut HashSet::new(), span)
        .expect_err("normalization work must remain bounded during inference");

    assert_eq!(error.message, "type normalization is too large");
    assert_eq!(error.primary.expect("normalization diagnostic").span, span);
}
