/**
 * @file Tree-sitter grammar for mal
 * @license MIT
 */

const PREC = {
  LOGICAL_OR: 1,
  LOGICAL_AND: 2,
  BIT_OR: 3,
  BIT_XOR: 4,
  BIT_AND: 5,
  EQUALITY: 6,
  RELATIONAL: 7,
  SHIFT: 8,
  ADDITIVE: 9,
  MULTIPLICATIVE: 10,
  INDEX: 11,
  PREFIX: 12,
  POSTFIX: 13,
};

module.exports = grammar({
  name: 'mal',

  extras: $ => [/[ \t\r\n]/, $.comment],

  word: $ => $.value_identifier,

  conflicts: $ => [
    [$.product_pattern, $.lambda_parameters],
    [$.pattern, $.value_name],
    [$.value_name, $.result_block],
    [$.value_name],
  ],

  rules: {
    source_file: $ => seq(
      repeat($.require_declaration),
      repeat($.top_level_item),
    ),

    require_declaration: $ => seq('require', field('path', $.symbol_literal), ';'),

    top_level_item: $ => choice(
      $.type_alias,
      $.extern_type_declaration,
      $.extern_value_declaration,
      $.generic_binding,
      $.binding,
    ),

    type_alias: $ => seq(
      field('name', $.type_identifier),
      optional($.type_parameters),
      '::',
      field('value', $._type),
      ';',
    ),

    extern_type_declaration: $ => seq(
      'extern',
      field('name', $.type_identifier),
      ';',
    ),

    extern_value_declaration: $ => seq(
      'extern',
      field('name', $.value_identifier),
      '::',
      field('type', $._type),
      ';',
    ),

    generic_binding: $ => seq(
      field('name', $.value_identifier),
      $.type_parameters,
      '::',
      field('type', $._type),
      ':=',
      field('value', $._expression),
      ';',
    ),

    binding: $ => seq(
      field('pattern', $.pattern),
      optional(seq('::', field('type', $._type))),
      ':=',
      field('value', $._expression),
      ';',
    ),

    type_parameters: $ => seq(
      '<',
      commaSep1(field('parameter', $.type_identifier)),
      '>',
    ),

    type_arguments: $ => seq('<', commaSep1($._type), '>'),

    _type: $ => prec.right(seq(
      $.atomic_type,
      optional(seq('->', $._type)),
    )),

    atomic_type: $ => choice(
      seq($.type_identifier, optional($.type_arguments)),
      seq('(', $._type, ')'),
      $.product_type,
      $.sum_type,
    ),

    product_type: $ => seq('(', $._type, ',', commaSep1($._type), ')'),

    sum_type: $ => seq('[', optional(commaSepMin2($._type)), ']'),

    pattern: $ => choice(
      $.value_identifier,
      $.wildcard,
      $.product_pattern,
    ),

    product_pattern: $ => seq('(', $.pattern, ',', commaSep1($.pattern), ')'),

    _expression: $ => choice(
      $.binary_expression,
      $.prefix_expression,
      $.postfix_expression,
      $.primary_expression,
    ),

    primary_expression: $ => choice(
      $.literal,
      $.value_name,
      $.unit_expression,
      $.parenthesized_expression,
      $.product_expression,
      $.unit_application,
      $.lambda_expression,
      $.block,
      $.result_block,
      $.if_expression,
      $.when_expression,
    ),

    value_name: $ => seq($.value_identifier, optional($.type_arguments)),

    unit_expression: _ => seq('(', ')'),

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    product_expression: $ => seq(
      '(',
      $._expression,
      ',',
      commaSep1($._expression),
      ')',
    ),

    unit_application: $ => seq('[', $._expression, ']'),

    lambda_expression: $ => seq(
      $.lambda_parameters,
      '->',
      field('body', $._expression),
    ),

    lambda_parameters: $ => seq(
      '(',
      optional(commaSep1(field('parameter', $.pattern))),
      ')',
    ),

    result_block: $ => seq(
      '[',
      commaSep1(field('result', $.value_identifier)),
      ']',
      '=>',
      field('body', $._expression),
    ),

    block: $ => seq(
      '{',
      repeat(choice($.binding, $.expression_statement)),
      optional(field('value', $._expression)),
      '}',
    ),

    expression_statement: $ => seq($._expression, ';'),

    if_expression: $ => seq(
      'if',
      '(',
      field('condition', $._expression),
      ')',
      'then',
      field('consequence', $._expression),
      'else',
      field('alternative', $._expression),
    ),

    when_expression: $ => seq(
      'when',
      '(',
      field('condition', $._expression),
      ')',
      field('consequence', $._expression),
    ),

    postfix_expression: $ => prec.left(PREC.POSTFIX, seq(
      $.primary_expression,
      repeat1($.postfix_suffix),
    )),

    postfix_suffix: $ => choice(
      $.call_suffix,
      $.receiver_suffix,
      $.continuation_suffix,
      $.conversion_suffix,
    ),

    call_suffix: $ => seq('(', optional(commaSep1($._expression)), ')'),

    receiver_suffix: $ => seq(
      '.',
      field('function', $.value_identifier),
      optional($.type_arguments),
      '(',
      optional(commaSep1($._expression)),
      ')',
    ),

    continuation_suffix: $ => seq(
      '[',
      optional(commaSep1($._expression)),
      ']',
    ),

    conversion_suffix: $ => seq('.', field('type', $.conversion_type)),

    conversion_type: _ => choice(
      'i8', 'i16', 'i32', 'i64',
      'u8', 'u16', 'u32', 'u64',
      'f32', 'f64', 'bytes', 'usize',
    ),

    prefix_expression: $ =>
      prec.right(PREC.PREFIX, seq(choice('-', '!', '~', '*', '#'), $._expression)),

    binary_expression: $ => choice(
      binaryLeft(PREC.LOGICAL_OR, $._expression, '||'),
      binaryLeft(PREC.LOGICAL_AND, $._expression, '&&'),
      binaryLeft(PREC.BIT_OR, $._expression, '|'),
      binaryLeft(PREC.BIT_XOR, $._expression, '^'),
      binaryLeft(PREC.BIT_AND, $._expression, '&'),
      binaryNonAssociative(PREC.EQUALITY, $._expression, choice('==', '!=')),
      binaryNonAssociative(PREC.RELATIONAL, $._expression, choice('<', '<=', '>', '>=')),
      binaryLeft(PREC.SHIFT, $._expression, choice('<<', '>>')),
      binaryLeft(PREC.ADDITIVE, $._expression, choice('+', '-')),
      binaryLeft(PREC.MULTIPLICATIVE, $._expression, choice('*', '/', '%')),
      binaryNonAssociative(PREC.INDEX, $._expression, '#'),
    ),

    literal: $ => choice(
      $.boolean_literal,
      $.float_literal,
      $.integer_literal,
      $.symbol_literal,
      $.byte_literal,
    ),

    boolean_literal: _ => choice('true', 'false'),

    float_literal: _ => token(choice(
      /[0-9](?:_?[0-9])*(?:\.[0-9](?:_?[0-9])*(?:[eE][+-]?[0-9](?:_?[0-9])*)?|[eE][+-]?[0-9](?:_?[0-9])*)?(?:f32|f64)/,
      /[0-9](?:_?[0-9])*(?:\.[0-9](?:_?[0-9])*(?:[eE][+-]?[0-9](?:_?[0-9])*)?|[eE][+-]?[0-9](?:_?[0-9])*)/,
    )),

    integer_literal: _ => token(choice(
      /0x[0-9A-Fa-f](?:_?[0-9A-Fa-f])*(?:(?:i|u)(?:8|16|32|64)|bytes|usize)?/,
      /0b[01](?:_?[01])*(?:(?:i|u)(?:8|16|32|64)|bytes|usize)?/,
      /[0-9](?:_?[0-9])*(?:(?:i|u)(?:8|16|32|64)|bytes|usize)?/,
    )),

    symbol_literal: _ => token(/"(?:[^"\\\r\n]|\\(?:[\\"nrt0]|x[0-9A-Fa-f]{2}))*"/),

    byte_literal: _ => token(/'(?:[\x20-\x26\x28-\x5b\x5d-\x7e]|\\(?:[\\'nrt0]|x[0-9A-Fa-f]{2}))'/),

    value_identifier: _ => /_?[a-z][A-Za-z0-9]*/,
    type_identifier: _ => /_?[A-Z][A-Za-z0-9]*/,
    wildcard: _ => '_',
    comment: _ => token(seq('//', /.*/)),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

function commaSepMin2(rule) {
  return seq(rule, ',', commaSep1(rule));
}

function binaryLeft(precedence, expression, operator) {
  return prec.left(precedence, seq(
    field('left', expression),
    field('operator', operator),
    field('right', expression),
  ));
}

function binaryNonAssociative(precedence, expression, operator) {
  return prec.left(precedence, seq(
    field('left', expression),
    field('operator', operator),
    field('right', expression),
  ));
}
