use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern};
use crate::check::ast as checked;

impl Lowerer {
    pub(super) fn lower_region_view(
        &mut self,
        range: &checked::Expression,
        callback: &checked::Expression,
        element: &checked::Type,
        expression: &checked::Expression,
    ) -> Expression {
        let range = self.lower_expression(range);
        let callback = self.lower_expression(callback);
        let range_id = self.temporary();
        let callback_id = self.temporary();
        let region_id = self.temporary();
        let region_type = checked::Type::Region(element.clone().into());
        let region = Expression {
            kind: ExpressionKind::Memory {
                primitive: checked::MemoryPrimitive::FormRegion,
                operands: vec![self.reference(range_id, range.ty.clone(), range.span)],
            },
            ty: region_type.clone(),
            span: expression.span,
        };
        let call = Expression {
            kind: ExpressionKind::Call {
                callee: Box::new(self.reference(callback_id, callback.ty.clone(), callback.span)),
                argument: Box::new(self.reference(region_id, region_type, expression.span)),
            },
            ty: expression.ty.clone(),
            span: expression.span,
        };
        let with_region = self.let_expression(
            Pattern::Binding {
                id: region_id,
                ty: region.ty.clone(),
            },
            region,
            call,
            expression.span,
        );
        let with_callback = self.let_expression(
            Pattern::Binding {
                id: callback_id,
                ty: callback.ty.clone(),
            },
            callback,
            with_region,
            expression.span,
        );
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern: Pattern::Binding {
                        id: range_id,
                        ty: range.ty.clone(),
                    },
                    value: range,
                    span: expression.span,
                }),
                body: Box::new(with_callback),
            },
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }
}
