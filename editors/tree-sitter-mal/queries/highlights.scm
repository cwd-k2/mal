(comment) @comment

(symbol_literal) @string
(byte_literal) @constant.character
(float_literal) @constant.numeric.float
(integer_literal) @constant.numeric.integer
(boolean_literal) @constant.builtin.boolean

(type_identifier) @type
(conversion_type) @type.builtin
(shape_atom) @type.builtin
(wildcard) @variable.builtin
(value_identifier) @variable

(type_alias name: (type_identifier) @type.definition)
(extern_type_declaration name: (type_identifier) @type.definition)
(extern_value_declaration name: (value_identifier) @function)
(generic_binding name: (value_identifier) @function)
(binding
  pattern: (pattern (value_identifier) @function)
  type: (atomic_type)
  type: (atomic_type))
(receiver_suffix function: (value_identifier) @function.method)

(type_parameters parameter: (type_identifier) @type.parameter)
(lambda_parameters parameter: (pattern (value_identifier) @variable.parameter))
(result_block result: (value_identifier) @variable.parameter)

((postfix_expression
  (primary_expression
    (value_name
      (value_identifier) @function))
  (postfix_suffix
    (call_suffix))))

["require" "extern"] @keyword
["if" "then" "else" "when"] @keyword.control

["::" ":=" "->" "=>"] @operator
["+" "-" "*" "/" "%" "<" "<=" ">" ">=" "==" "!=" "&&" "||" "&" "|" "^" "!" "~" "#" "<<" ">>"] @operator

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";"] @punctuation.delimiter
"." @punctuation.delimiter
