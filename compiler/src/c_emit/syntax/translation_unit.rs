use super::{AggregateDefinition, Comment, Declaration, Directive, FunctionDefinition};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum UnitItem {
    Aggregate(AggregateDefinition),
    Comment(Comment),
    Declaration(Declaration),
    Directive(Directive),
    Function(FunctionDefinition),
    BlankLine,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::c_emit) struct TranslationUnit {
    items: Vec<UnitItem>,
}

impl TranslationUnit {
    pub(in crate::c_emit) fn new(items: impl IntoIterator<Item = UnitItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn push(&mut self, item: impl Into<UnitItem>) {
        self.items.push(item.into());
    }

    pub(in crate::c_emit) fn extend(&mut self, other: Self) {
        self.items.extend(other.items);
    }

    pub(in crate::c_emit) fn blank_line(&mut self) {
        if !self.items.is_empty() && !matches!(self.items.last(), Some(UnitItem::BlankLine)) {
            self.items.push(UnitItem::BlankLine);
        }
    }

    pub(in crate::c_emit) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = String::new();
        for item in &self.items {
            match item {
                UnitItem::Aggregate(definition) => output.push_str(&definition.render()),
                UnitItem::Comment(comment) => output.push_str(&comment.render()),
                UnitItem::Declaration(declaration) => output.push_str(&declaration.render()),
                UnitItem::Directive(directive) => output.push_str(&directive.render()),
                UnitItem::Function(definition) => output.push_str(&definition.render()),
                UnitItem::BlankLine => output.push('\n'),
            }
        }
        output
    }
}

impl From<AggregateDefinition> for UnitItem {
    fn from(value: AggregateDefinition) -> Self {
        Self::Aggregate(value)
    }
}

impl From<Comment> for UnitItem {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}

impl From<Declaration> for UnitItem {
    fn from(value: Declaration) -> Self {
        Self::Declaration(value)
    }
}

impl From<Directive> for UnitItem {
    fn from(value: Directive) -> Self {
        Self::Directive(value)
    }
}

impl From<FunctionDefinition> for UnitItem {
    fn from(value: FunctionDefinition) -> Self {
        Self::Function(value)
    }
}

#[cfg(test)]
mod tests {
    use super::TranslationUnit;
    use crate::c_emit::syntax::{Declaration, Directive, TypeName};

    #[test]
    fn renders_items_and_owned_section_spacing() {
        let mut unit = TranslationUnit::default();
        unit.push(Directive::IncludeSystem("stdint.h".into()));
        unit.blank_line();
        unit.blank_line();
        unit.push(Declaration::type_alias(TypeName::named("uint8_t"), "Byte"));

        assert_eq!(
            unit.render(),
            "#include <stdint.h>\n\ntypedef uint8_t Byte;\n"
        );
    }
}
