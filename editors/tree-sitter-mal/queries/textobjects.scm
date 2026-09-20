(comment)+ @comment.around
(comment) @comment.inside

(lambda_expression) @function.around
(lambda_expression body: (_) @function.inside)
(generic_binding) @function.around
(generic_binding value: (_) @function.inside)

(lambda_parameters (pattern) @parameter.inside)
(call_suffix (_) @parameter.inside)
(receiver_suffix (_) @parameter.inside)
