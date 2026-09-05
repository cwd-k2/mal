use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::source::Span;

pub mod ast;
mod expression;

use self::ast::{
    BOOL_TYPE, BYTE_AT_VALUE, BYTE_LENGTH_VALUE, ExternalOperationId, FALSE_VALUE, INT8_TYPE,
    INT16_TYPE, INT32_TYPE, INT64_TYPE, LambdaId, Program, STRING_TYPE, TRUE_VALUE, TypeBinding,
    TypeId, TypeReference, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
    ValueBinding, ValueId, ValueOwner,
};

pub fn resolve(program: &crate::ast::Program) -> Result<Program, Diagnostic> {
    Resolver::new(program.span).resolve_program(program)
}

#[derive(Clone)]
struct ExternalBinding {
    id: ExternalOperationId,
}

struct Resolver {
    types: HashMap<String, TypeBinding>,
    externals: HashMap<String, ExternalBinding>,
    value_scopes: Vec<HashMap<String, ValueBinding>>,
    current_lambda: Option<LambdaId>,
    recursive_lambda: Option<(LambdaId, ValueId)>,
    next_type: u32,
    next_value: u32,
    next_external: u32,
    next_lambda: u32,
    synthetic_span: Span,
}

impl Resolver {
    fn new(program_span: Span) -> Self {
        let synthetic_span = Span::new(
            program_span.file(),
            program_span.start(),
            program_span.start(),
        );
        let mut resolver = Self {
            types: HashMap::new(),
            externals: HashMap::new(),
            value_scopes: vec![HashMap::new()],
            current_lambda: None,
            recursive_lambda: None,
            next_type: 11,
            next_value: 4,
            next_external: 0,
            next_lambda: 0,
            synthetic_span,
        };
        resolver.add_predefined_type("Unit", UNIT_TYPE);
        resolver.add_predefined_type("Int8", INT8_TYPE);
        resolver.add_predefined_type("Int16", INT16_TYPE);
        resolver.add_predefined_type("Int32", INT32_TYPE);
        resolver.add_predefined_type("Int64", INT64_TYPE);
        resolver.add_predefined_type("UInt8", UINT8_TYPE);
        resolver.add_predefined_type("UInt16", UINT16_TYPE);
        resolver.add_predefined_type("UInt32", UINT32_TYPE);
        resolver.add_predefined_type("UInt64", UINT64_TYPE);
        resolver.add_predefined_type("Bool", BOOL_TYPE);
        resolver.add_predefined_type("String", STRING_TYPE);
        resolver.add_predefined_value("false", FALSE_VALUE);
        resolver.add_predefined_value("true", TRUE_VALUE);
        resolver.add_predefined_value("byteLength", BYTE_LENGTH_VALUE);
        resolver.add_predefined_value("byteAt", BYTE_AT_VALUE);
        resolver
    }

    fn resolve_program(mut self, program: &crate::ast::Program) -> Result<Program, Diagnostic> {
        self.predeclare_unit_names(program)?;
        let mut items = Vec::with_capacity(program.items.len());
        for item in &program.items {
            items.push(self.resolve_top_item(item)?);
        }
        Ok(Program {
            items,
            span: program.span,
        })
    }

    fn predeclare_unit_names(&mut self, program: &crate::ast::Program) -> Result<(), Diagnostic> {
        for item in &program.items {
            match &item.kind {
                crate::ast::TopItem::TypeAlias { name, .. }
                | crate::ast::TopItem::ExternalType { name } => {
                    if self.types.contains_key(&name.text) {
                        return Err(self.duplicate(name, "type"));
                    }
                    let binding = TypeBinding {
                        id: TypeId(self.next_type),
                        name: name.clone(),
                    };
                    self.next_type += 1;
                    self.types.insert(name.text.clone(), binding);
                }
                crate::ast::TopItem::ExternalOperation { name, .. } => {
                    if self.externals.contains_key(&name.text)
                        || self.value_scopes[0].contains_key(&name.text)
                    {
                        return Err(self.duplicate(name, "top-level value"));
                    }
                    let binding = ExternalBinding {
                        id: ExternalOperationId(self.next_external),
                    };
                    self.next_external += 1;
                    self.externals.insert(name.text.clone(), binding);
                }
                crate::ast::TopItem::Binding(_) => {}
            }
        }
        Ok(())
    }

    fn resolve_top_item(
        &mut self,
        item: &crate::ast::Node<crate::ast::TopItem>,
    ) -> Result<crate::ast::Node<ast::TopItem>, Diagnostic> {
        let kind = match &item.kind {
            crate::ast::TopItem::TypeAlias { name, value } => ast::TopItem::TypeAlias {
                binding: self.type_binding(name)?,
                value: self.resolve_type(value)?,
            },
            crate::ast::TopItem::ExternalType { name } => ast::TopItem::ExternalType {
                binding: self.type_binding(name)?,
            },
            crate::ast::TopItem::ExternalOperation { name, ty } => {
                let external = self
                    .externals
                    .get(&name.text)
                    .expect("external declarations are predeclared");
                ast::TopItem::ExternalOperation {
                    id: external.id,
                    name: name.clone(),
                    ty: self.resolve_type(ty)?,
                }
            }
            crate::ast::TopItem::Binding(binding) => {
                ast::TopItem::Binding(self.resolve_binding(binding, ValueOwner::TopLevel)?)
            }
        };
        Ok(crate::ast::Node::new(kind, item.span))
    }

    fn resolve_type(
        &self,
        ty: &crate::ast::Node<crate::ast::TypeExpression>,
    ) -> Result<crate::ast::Node<ast::TypeExpression>, Diagnostic> {
        let kind = match &ty.kind {
            crate::ast::TypeExpression::Named(name) => {
                ast::TypeExpression::Named(self.type_reference(name)?)
            }
            crate::ast::TypeExpression::Unit => ast::TypeExpression::Unit,
            crate::ast::TypeExpression::Parenthesized(inner) => {
                ast::TypeExpression::Parenthesized(Box::new(self.resolve_type(inner)?))
            }
            crate::ast::TypeExpression::Product(elements) => ast::TypeExpression::Product(
                elements
                    .iter()
                    .map(|element| self.resolve_type(element))
                    .collect::<Result<_, _>>()?,
            ),
            crate::ast::TypeExpression::Sum(members) => ast::TypeExpression::Sum(
                members
                    .iter()
                    .map(|member| self.resolve_type(member))
                    .collect::<Result<_, _>>()?,
            ),
            crate::ast::TypeExpression::Function { parameter, result } => {
                ast::TypeExpression::Function {
                    parameter: Box::new(self.resolve_type(parameter)?),
                    result: Box::new(self.resolve_type(result)?),
                }
            }
        };
        Ok(crate::ast::Node::new(kind, ty.span))
    }

    fn resolve_binding(
        &mut self,
        binding: &crate::ast::Binding,
        owner: ValueOwner,
    ) -> Result<ast::Binding, Diagnostic> {
        let annotation = binding
            .annotation
            .as_ref()
            .map(|ty| self.resolve_type(ty))
            .transpose()?;

        if annotation.is_some()
            && let crate::ast::Pattern::Name(name) = &binding.pattern.kind
            && let crate::ast::Expression::Lambda(lambda) = &binding.value.kind
        {
            let declared = self.declare_value(name, owner)?;
            let value = crate::ast::Node::new(
                ast::Expression::Lambda(
                    self.resolve_lambda_with_self(lambda, Some(declared.clone()))?,
                ),
                binding.value.span,
            );
            let pattern =
                crate::ast::Node::new(ast::Pattern::Binding(declared), binding.pattern.span);
            return Ok(ast::Binding {
                pattern,
                annotation,
                value,
            });
        }

        let value = self.resolve_expression(&binding.value)?;
        let pattern = self.declare_pattern(&binding.pattern, owner)?;
        Ok(ast::Binding {
            pattern,
            annotation,
            value,
        })
    }

    fn declare_pattern(
        &mut self,
        pattern: &crate::ast::Node<crate::ast::Pattern>,
        owner: ValueOwner,
    ) -> Result<crate::ast::Node<ast::Pattern>, Diagnostic> {
        let kind = match &pattern.kind {
            crate::ast::Pattern::Name(name) => {
                ast::Pattern::Binding(self.declare_value(name, owner)?)
            }
            crate::ast::Pattern::Wildcard => ast::Pattern::Wildcard,
            crate::ast::Pattern::Product(elements) => ast::Pattern::Product(
                elements
                    .iter()
                    .map(|element| self.declare_pattern(element, owner))
                    .collect::<Result<_, _>>()?,
            ),
        };
        Ok(crate::ast::Node::new(kind, pattern.span))
    }

    fn declare_value(
        &mut self,
        name: &crate::ast::Name,
        owner: ValueOwner,
    ) -> Result<ValueBinding, Diagnostic> {
        if self
            .value_scopes
            .last()
            .expect("value scope")
            .contains_key(&name.text)
            || (owner == ValueOwner::TopLevel && self.externals.contains_key(&name.text))
        {
            return Err(self.duplicate(name, "value"));
        }
        let binding = ValueBinding {
            id: ValueId(self.next_value),
            name: name.clone(),
            owner,
        };
        self.next_value += 1;
        self.value_scopes
            .last_mut()
            .expect("value scope")
            .insert(name.text.clone(), binding.clone());
        Ok(binding)
    }

    fn type_binding(&self, name: &crate::ast::Name) -> Result<TypeBinding, Diagnostic> {
        self.types
            .get(&name.text)
            .cloned()
            .ok_or_else(|| self.unknown(name, "type"))
    }

    fn type_reference(&self, name: &crate::ast::Name) -> Result<TypeReference, Diagnostic> {
        Ok(TypeReference {
            id: self.type_binding(name)?.id,
            name: name.clone(),
        })
    }

    fn lookup_value(&self, text: &str) -> Option<ValueBinding> {
        self.value_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(text).cloned())
    }

    fn push_scope(&mut self) {
        self.value_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.value_scopes.pop().expect("a nested value scope");
    }

    fn allocate_lambda(&mut self) -> LambdaId {
        let id = LambdaId(self.next_lambda);
        self.next_lambda += 1;
        id
    }

    fn local_owner(&self, span: Span) -> Result<ValueOwner, Diagnostic> {
        self.current_lambda.map(ValueOwner::Lambda).ok_or_else(|| {
            Diagnostic::error("local binding outside a lambda is not supported")
                .with_primary(span, "this binding has no owning lambda")
        })
    }

    fn add_predefined_type(&mut self, text: &str, id: TypeId) {
        self.types.insert(
            text.into(),
            TypeBinding {
                id,
                name: self.synthetic_name(text),
            },
        );
    }

    fn add_predefined_value(&mut self, text: &str, id: ValueId) {
        let name = self.synthetic_name(text);
        self.value_scopes[0].insert(
            text.into(),
            ValueBinding {
                id,
                name,
                owner: ValueOwner::Predefined,
            },
        );
    }

    fn synthetic_name(&self, text: &str) -> crate::ast::Name {
        crate::ast::Name {
            text: text.into(),
            span: self.synthetic_span,
        }
    }

    fn duplicate(&self, name: &crate::ast::Name, category: &str) -> Diagnostic {
        Diagnostic::error(format!("duplicate {category} `{}`", name.text))
            .with_primary(name.span, "already declared in this scope")
    }

    fn unknown(&self, name: &crate::ast::Name, category: &str) -> Diagnostic {
        Diagnostic::error(format!("unknown {category} `{}`", name.text))
            .with_primary(name.span, format!("this {category} is not in scope"))
    }
}
