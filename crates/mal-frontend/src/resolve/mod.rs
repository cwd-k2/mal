use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

pub mod ast;
mod expression;
mod files;
mod predefined;
mod scope;

pub use self::predefined::{
    COPY_VALUE, FILL_VALUE, FROM_VALUE, GET_VALUE, INTO_VALUE, MAKE_VALUE, NEW_VALUE, PUT_VALUE,
    TYPES as PREDEFINED_TYPES, VALUES as PREDEFINED_VALUES,
};

use self::ast::{
    ExternalOperationId, LambdaId, Program, TypeBinding, ValueBinding, ValueId, ValueOwner,
};

/// Resolves one parsed file. `require` declarations are not followed; use `resolve_graph` for several files.
pub fn resolve(program: &mal_syntax::ast::Program) -> Result<Program, Diagnostic> {
    Resolver::new(program.span).resolve_program(program)
}

/// Resolves the files of `graph`, given in graph order, as one program. Public names flow only along direct requirements.
pub fn resolve_graph(
    graph: &mal_syntax::source::SourceGraph,
    programs: &[mal_syntax::ast::Program],
) -> Result<Program, Diagnostic> {
    files::resolve(graph, programs)
}

#[derive(Clone)]
struct ExternalBinding {
    id: ExternalOperationId,
    binding: ValueBinding,
    lambda_id: LambdaId,
}

struct LambdaFrame {
    id: LambdaId,
    captures: Vec<ast::Capture>,
    captured_sources: HashMap<ValueId, ValueBinding>,
}

struct Resolver {
    types: HashMap<String, TypeBinding>,
    externals: HashMap<String, ExternalBinding>,
    value_scopes: Vec<HashMap<String, ValueBinding>>,
    current_lambda: Option<LambdaId>,
    recursive_lambda: Option<(LambdaId, ValueId)>,
    lambda_frames: Vec<LambdaFrame>,
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
            lambda_frames: Vec::new(),
            next_type: predefined::first_source_type_id(),
            next_value: predefined::first_source_value_id(),
            next_external: 0,
            next_lambda: 0,
            synthetic_span,
        };
        for entry in PREDEFINED_TYPES {
            resolver.add_predefined_type(entry.name, entry.id);
        }
        for entry in PREDEFINED_VALUES {
            resolver.add_predefined_value(entry.name, entry.id);
        }
        resolver
    }

    fn resolve_program(
        mut self,
        program: &mal_syntax::ast::Program,
    ) -> Result<Program, Diagnostic> {
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

    fn begin_file(&mut self, span: Span) {
        self.types.clear();
        self.externals.clear();
        self.value_scopes.clear();
        self.value_scopes.push(HashMap::new());
        self.current_lambda = None;
        self.recursive_lambda = None;
        self.lambda_frames.clear();
        self.synthetic_span = Span::new(span.file(), span.start(), span.start());
        for entry in PREDEFINED_TYPES {
            self.add_predefined_type(entry.name, entry.id);
        }
        for entry in PREDEFINED_VALUES {
            self.add_predefined_value(entry.name, entry.id);
        }
    }

    fn resolve_top_item(
        &mut self,
        item: &mal_syntax::ast::Node<mal_syntax::ast::TopItem>,
    ) -> Result<mal_syntax::ast::Node<ast::TopItem>, Diagnostic> {
        let kind = match &item.kind {
            mal_syntax::ast::TopItem::TypeAlias { name, value } => ast::TopItem::TypeAlias {
                binding: self.type_binding(name)?,
                value: self.resolve_type(value)?,
            },
            mal_syntax::ast::TopItem::GenericTypeAlias {
                name,
                parameters,
                value,
            } => {
                let (bindings, shadowed) = self.push_type_parameters(parameters)?;
                let resolved = self.resolve_type(value);
                self.pop_type_parameters(&bindings, shadowed);
                ast::TopItem::GenericTypeAlias {
                    binding: self.type_binding(name)?,
                    parameters: bindings,
                    value: resolved?,
                }
            }
            mal_syntax::ast::TopItem::ExternalType { name } => ast::TopItem::ExternalType {
                binding: self.type_binding(name)?,
            },
            mal_syntax::ast::TopItem::ExternalOperation { name, ty } => {
                let external = self
                    .externals
                    .get(&name.text)
                    .expect("external declarations are predeclared");
                ast::TopItem::ExternalOperation {
                    id: external.id,
                    binding: external.binding.clone(),
                    lambda_id: external.lambda_id,
                    ty: self.resolve_type(ty)?,
                }
            }
            mal_syntax::ast::TopItem::Binding(binding) => {
                ast::TopItem::Binding(self.resolve_binding(binding, ValueOwner::TopLevel)?)
            }
            mal_syntax::ast::TopItem::GenericBinding {
                name,
                parameters,
                annotation,
                value,
            } => {
                let binding = self.declare_value(name, ValueOwner::TopLevel)?;
                let (parameter_bindings, shadowed) = self.push_type_parameters(parameters)?;
                let resolved = (|| {
                    let annotation = self.resolve_type(annotation)?;
                    let value = if let mal_syntax::ast::Expression::Lambda(lambda) = &value.kind {
                        mal_syntax::ast::Node::new(
                            ast::Expression::Lambda(
                                self.resolve_lambda_with_self(lambda, Some(binding.clone()))?,
                            ),
                            value.span,
                        )
                    } else {
                        self.resolve_expression(value)?
                    };
                    Ok((annotation, value))
                })();
                self.pop_type_parameters(&parameter_bindings, shadowed);
                let (annotation, value) = resolved?;
                ast::TopItem::GenericBinding {
                    binding,
                    parameters: parameter_bindings,
                    annotation,
                    value,
                }
            }
        };
        Ok(mal_syntax::ast::Node::new(kind, item.span))
    }

    fn resolve_type(
        &self,
        ty: &mal_syntax::ast::Node<mal_syntax::ast::TypeExpression>,
    ) -> Result<mal_syntax::ast::Node<ast::TypeExpression>, Diagnostic> {
        let kind = match &ty.kind {
            mal_syntax::ast::TypeExpression::Named(name) => {
                ast::TypeExpression::Named(self.type_reference(name)?)
            }
            mal_syntax::ast::TypeExpression::Application {
                constructor,
                arguments,
            } => ast::TypeExpression::Application {
                constructor: self.type_constructor_reference(constructor)?,
                arguments: arguments
                    .iter()
                    .map(|argument| self.resolve_type(argument))
                    .collect::<Result<_, _>>()?,
            },
            mal_syntax::ast::TypeExpression::Unit => ast::TypeExpression::Unit,
            mal_syntax::ast::TypeExpression::Parenthesized(inner) => {
                ast::TypeExpression::Parenthesized(Box::new(self.resolve_type(inner)?))
            }
            mal_syntax::ast::TypeExpression::Product(elements) => ast::TypeExpression::Product(
                elements
                    .iter()
                    .map(|element| self.resolve_type(element))
                    .collect::<Result<_, _>>()?,
            ),
            mal_syntax::ast::TypeExpression::Sum(members) => ast::TypeExpression::Sum(
                members
                    .iter()
                    .map(|member| self.resolve_type(member))
                    .collect::<Result<_, _>>()?,
            ),
            mal_syntax::ast::TypeExpression::Function { parameter, result } => {
                ast::TypeExpression::Function {
                    parameter: Box::new(self.resolve_type(parameter)?),
                    result: Box::new(self.resolve_type(result)?),
                }
            }
        };
        Ok(mal_syntax::ast::Node::new(kind, ty.span))
    }

    fn push_type_parameters(
        &mut self,
        parameters: &[mal_syntax::ast::Name],
    ) -> Result<(Vec<TypeBinding>, Vec<Option<TypeBinding>>), Diagnostic> {
        let mut bindings = Vec::with_capacity(parameters.len());
        let mut shadowed = Vec::with_capacity(parameters.len());
        let mut names = std::collections::HashSet::new();
        for name in parameters {
            if !names.insert(name.text.clone()) {
                self.pop_type_parameters(&bindings, shadowed);
                return Err(Diagnostic::error("duplicate type parameter")
                    .with_primary(name.span, "this parameter is declared more than once"));
            }
            let binding = TypeBinding {
                id: ast::TypeId(self.next_type),
                name: name.clone(),
            };
            self.next_type += 1;
            shadowed.push(self.types.insert(name.text.clone(), binding.clone()));
            bindings.push(binding);
        }
        Ok((bindings, shadowed))
    }

    fn pop_type_parameters(
        &mut self,
        parameters: &[TypeBinding],
        shadowed: Vec<Option<TypeBinding>>,
    ) {
        for (parameter, previous) in parameters.iter().zip(shadowed) {
            self.types.remove(&parameter.name.text);
            if let Some(previous) = previous {
                self.types.insert(parameter.name.text.clone(), previous);
            }
        }
    }

    fn resolve_binding(
        &mut self,
        binding: &mal_syntax::ast::Binding,
        owner: ValueOwner,
    ) -> Result<ast::Binding, Diagnostic> {
        let annotation = binding
            .annotation
            .as_ref()
            .map(|ty| self.resolve_type(ty))
            .transpose()?;

        if annotation.is_some()
            && let mal_syntax::ast::Pattern::Name(name) = &binding.pattern.kind
            && let mal_syntax::ast::Expression::Lambda(lambda) = &binding.value.kind
        {
            let declared = self.declare_value(name, owner)?;
            let value = mal_syntax::ast::Node::new(
                ast::Expression::Lambda(
                    self.resolve_lambda_with_self(lambda, Some(declared.clone()))?,
                ),
                binding.value.span,
            );
            let pattern =
                mal_syntax::ast::Node::new(ast::Pattern::Binding(declared), binding.pattern.span);
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
}
