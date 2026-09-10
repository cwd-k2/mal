# v0.5 conformance matrix

Status: Current v0.5 evidence

この文書は[`spec/`](../spec/)の規範とreference compilerの検証先を対応付ける。規則そのものは
`spec/`、test layerと実行commandは[test policy](testing.md)を正とする。

表の`P`はpositive、`N`はnegative、`E`はedge、`X`はbackend artifactをcompileして実行するnative testを表す。
`—`は、その節が言語外の範囲や文書上の責務を定め、該当する実行時挙動を持たないことを表す。
test名はRustのtest function名であり、同じ行のfileに属する。

## 言語の範囲と型

| 規範 | P / N / E | X |
|---|---|---|
| [`scope`: malが持つもの](../spec/scope.md#mal-が持つもの) | 各機能は以下の対応行で検証 | `examples/`のchecked-in program（`compiler/tests/driver.rs`） |
| [`scope`: 持たないもの](../spec/scope.md#mal-が持たないもの) | `rejects_unknown_names_and_reserved_top_level_redefinitions`（`compiler/tests/resolve.rs`）、`rejects_non_associative_operator_chains`（`compiler/tests/parser.rs`） | — |
| [`scope`: named data](../spec/scope.md#named-data) | P/E: `expands_aliases_and_compares_types_structurally`、`checks_sum_injection_payload_and_index`（`compiler/tests/check.rs`） | `branches_over_bool_and_unmanaged_sums_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`scope`: memoryとmutable data](../spec/scope.md#memory-と-mutable-data) | P/N: `checks_nominal_external_opaque_types`（`compiler/tests/check.rs`） | `transfers_external_opaque_values_through_the_public_c_abi`（`compiler/tests/driver/artifacts.rs`） |
| [`scope`: standard libraryとfile](../spec/scope.md#standard-library-と-file) | require grammarとpath rejection（`compiler/tests/parser.rs`、`compiler/tests/driver.rs`） | `build_compiles_required_host_inputs_and_produces_an_executable`（`compiler/tests/driver.rs`） |
| [`scope`: 設計原則](../spec/scope.md#設計原則) | 以下の型・`extern` ABI対応行で検証 | 以下のABI testで検証 |
| [`types`: scalarと型の構成](../spec/types.md#型の構成) | P/N/E: `checks_all_fixed_width_literal_boundaries_and_byte_literals`、`checks_float_arithmetic_comparison_and_negation`（`compiler/tests/check.rs`） | `emit_header_writes_a_standalone_host_interface`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: Symbol](../spec/types.md#symbol) | P/N: `checks_symbol_literals_as_immutable_bytes`、`rejects_unsupported_or_mistyped_symbol_operations`（`compiler/tests/check.rs`） | `owns_flat_symbols_across_direct_llvm_calls`（`compiler/tests/driver/artifacts.rs`） |
| [`memory`: Ptr、storage幅、primitive](../spec/memory.md) | P/N/E: `resolves_memory_primitives_and_the_ptr_type`、`resolves_the_type_in_a_type_qualified_primitive`（`compiler/tests/resolve.rs`）、`checks_ptr_extern_signatures_and_memory_primitives`、`checks_memory_primitives_for_every_supported_value_type`、`gives_every_memory_function_a_first_class_function_type`、`checks_storage_sizes_for_scalar_and_ptr_types`、`rejects_storage_sizes_without_a_memory_representation`、`rejects_mistyped_memory_operations`（`compiler/tests/check.rs`） | `accesses_unaligned_scalar_and_pointer_storage_through_llvm`、`owns_symbols_nested_in_products_through_llvm`、`calls_first_class_memory_functions_through_llvm`（`compiler/tests/driver/artifacts.rs`）、`pointer-tree` example（`compiler/tests/driver.rs`） |
| [`types`: Unit](../spec/types.md#unit) | P/E: `checks_function_application_and_zero_argument_unit_lowering`（`compiler/tests/check.rs`） | `build_compiles_required_host_inputs_and_produces_an_executable`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: 直積](../spec/types.md#直積) | P/N/E: `checks_products_destructuring_and_multiple_parameters`（`compiler/tests/check.rs`） | `constructs_and_resumes_unmanaged_products_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: 直和](../spec/types.md#直和) | P/N/E: `checks_postfix_application_and_sum_continuations`、`rejects_invalid_sum_continuations_and_constructors`（`compiler/tests/check.rs`）、`rejects_single_member_sums_and_trailing_commas`（`compiler/tests/parser.rs`） | `branches_over_bool_and_unmanaged_sums_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: predefined Bool](../spec/types.md#predefined-bool) | P/N/E: `checks_int32_and_bool_operator_families`（`compiler/tests/check.rs`）、`local_scope_can_shadow_predefined_and_outer_names`、`rejects_unknown_names_and_reserved_top_level_redefinitions`（`compiler/tests/resolve.rs`） | `preserves_short_circuit_effect_order_through_llvm`、`branches_over_bool_and_unmanaged_sums_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: transparent alias](../spec/types.md#transparent-alias) | P/N/E: `expands_aliases_and_compares_types_structurally`、`rejects_recursive_aliases_even_when_unused`（`compiler/tests/check.rs`） | `preserves_type_alias_names_as_backend_metadata`（`compiler/tests/core.rs`） |
| [`types`: 関数型](../spec/types.md#関数型) | P/E: `function_types_are_right_associative`（`compiler/tests/parser.rs`）、`checks_function_application_and_zero_argument_unit_lowering`（`compiler/tests/check.rs`） | `calls_escaping_closures_with_managed_captures_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`types`: external opaque type](../spec/types.md#external-opaque-type) | P/N: `checks_nominal_external_opaque_types`（`compiler/tests/check.rs`） | `transfers_external_opaque_values_through_the_public_c_abi`（`compiler/tests/driver/artifacts.rs`） |

## 式と実行意味論

| 規範 | P / N / E | X |
|---|---|---|
| [`expressions`: binding](../spec/expressions.md#binding) | P/N/E: `checks_products_destructuring_and_multiple_parameters`（`compiler/tests/check.rs`）、`local_bindings_enter_scope_only_after_their_initializer`（`compiler/tests/resolve.rs`） | `constructs_and_resumes_unmanaged_products_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: ラムダ](../spec/expressions.md#ラムダ) | P/N/E: `parses_parameters_and_lambda_body_items`、`accepts_block_results_with_or_without_a_terminal_semicolon`、`rejects_a_lambda_without_a_result_expression`（`compiler/tests/parser.rs`）、lexical capture推論群（`compiler/tests/resolve.rs`） | `calls_escaping_closures_with_managed_captures_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: 関数適用](../spec/expressions.md#関数適用) | P/E: `checks_function_application_and_zero_argument_unit_lowering`（`compiler/tests/check.rs`）、`evaluates_a_callee_before_its_argument_and_application`（`compiler/tests/anf.rs`） | `print-and-closure` example（`compiler/tests/driver.rs`） |
| [`expressions`: if](../spec/expressions.md#if) | P/N/E: `checks_if_condition_and_branch_types`（`compiler/tests/check.rs`）、`lowers_if_to_false_then_true_case_arms`（`compiler/tests/core.rs`） | `preserves_short_circuit_effect_order_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: 直和の除去](../spec/expressions.md#直和の除去) | P/N/E: `parses_postfix_and_unit_continuation_applications`（`compiler/tests/parser.rs`）、`checks_postfix_application_and_sum_continuations`、`rejects_invalid_sum_continuations_and_constructors`（`compiler/tests/check.rs`） | `branches_over_bool_and_unmanaged_sums_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: numeric literal](../spec/expressions.md#literal) | P/N/E: numeric lexer tests（`compiler/tests/lexer.rs`）、integer/float boundary tests（`compiler/tests/check.rs`） | `builds_every_integer_width_with_signed_and_unsigned_llvm_comparisons`、`builds_strict_float_arithmetic_and_nan_comparisons_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: byte literal](../spec/expressions.md#byte-literal) | P/N/E: `lexes_byte_literals_and_every_escape`、`rejects_malformed_byte_literals_at_the_lexer_boundary`（`compiler/tests/lexer.rs`） | `integer-and-byte` example（`compiler/tests/driver.rs`） |
| [`expressions`: primitive operator](../spec/expressions.md#primitive-operator) | P/N/E: integer、float、Bool、conversion tests（`compiler/tests/check.rs`） | integer、float、Bool、conversion testsとprecondition trap非生成（`compiler/tests/driver/artifacts.rs`） |
| [`expressions`: expression statement](../spec/expressions.md#expression-statement) | P/E: `lowers_lambda_statements_and_a_block_result_to_lets_and_a_result`（`compiler/tests/core.rs`）、`flattens_core_lets_without_losing_statement_order`（`compiler/tests/anf.rs`） | `builds_scalar_control_and_tail_calls_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: 評価戦略](../spec/execution.md#評価戦略) | P/E: operand、value、continuation、sum elimination、statement、product順序（`compiler/tests/anf.rs`） | `builds_scalar_control_and_tail_calls_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: scopeとclosure](../spec/execution.md#scope-と-closure) | P/N/E: capture resolution群（`compiler/tests/resolve.rs`）、closure conversion群（`compiler/tests/closure.rs`） | `calls_escaping_closures_with_managed_captures_through_llvm`、allocation trap（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: 再帰](../spec/execution.md#再帰) | P/N/E: recursion acceptance/rejection群（`compiler/tests/resolve.rs`、`compiler/tests/check.rs`） | `dispatches_all_recursive_closure_targets_through_llvm`、scalarおよびmanaged valueのdirect-tail-call tests（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: 整数](../spec/execution.md#整数) | P/N/E: `checks_integer_operators_for_every_fixed_width_type`（`compiler/tests/check.rs`） | wrapping、shift、division/remainder tests（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: 浮動小数点](../spec/execution.md#浮動小数点) | P/N/E: float literal/operator/conversion tests（`compiler/tests/check.rs`） | strict arithmetic、comparison、conversion tests（`compiler/tests/driver/artifacts.rs`）、`strict-float` example（`compiler/tests/driver.rs`） |
| [`execution`: trap](../spec/execution.md#trap) | storage failureとpreconditionの分類 | closureとSymbolのallocation failure、およびprecondition trap非生成（`compiler/tests/driver/artifacts.rs`） |
| [`execution`: core calculus](../spec/execution.md#core-calculus) | surface消去（`compiler/tests/core.rs`）、評価順序のANF化（`compiler/tests/anf.rs`） | 代表経路は各native test |

## Symbol、extern、C ABI

| 規範 | P / N / E | X |
|---|---|---|
| [`engrams`: authorityと境界](../spec/engrams.md) | P/N: transportable type、`Symbol.size` rejection、memory operation tests（`compiler/tests/check.rs`） | host copy、Symbol memory copy、process argument admission、`owns_flat_symbols_across_direct_llvm_calls`（`compiler/tests/driver/artifacts.rs`） |
| [`symbols`: 値](../spec/symbols.md#値) | P/E: `checks_symbol_literals_as_immutable_bytes`（`compiler/tests/check.rs`） | static/captured/copy tests、`owns_capturing_closure_environments_through_llvm`、`retains_only_active_managed_sum_payloads_through_llvm`、`retains_only_active_managed_sum_payloads_through_llvm`、`runs_managed_direct_self_tail_calls_through_llvm`（`compiler/tests/driver/artifacts.rs`） |
| [`symbols`: literal](../spec/symbols.md#literal) | P/N/E: Symbol literal lexer tests（`compiler/tests/lexer.rs`） | `owns_symbols_across_direct_llvm_calls`（`compiler/tests/driver/artifacts.rs`） |
| [`symbols`: operator](../spec/symbols.md#operator) | P/N/E: `parses_symbol_length_and_byte_access_with_access_precedence`、`rejects_chained_symbol_byte_access`（`compiler/tests/parser.rs`）、`checks_symbol_operators_and_byte_wise_equality`、`rejects_unsupported_or_mistyped_symbol_operations`（`compiler/tests/check.rs`） | `owns_symbols_across_direct_llvm_calls`、`balances_persistent_symbols_and_materializes_only_at_the_host_boundary`、precondition trap非生成、allocation failure tests（`compiler/tests/driver/artifacts.rs`） |
| [`symbols`: mutable bytesとの分離](../spec/symbols.md#mutable-bytesとの分離) | P/N: opaque typeとunsupported operation tests（`compiler/tests/check.rs`） | `symbol-round-trip` example（`compiler/tests/driver.rs`） |
| [`extern`: 目的とsource semantics](../spec/extern.md#目的) | P/N: `validates_extern_signatures_recursively`（`compiler/tests/check.rs`）、extern parse/resolve tests | host adapterを持つchecked-in example（`compiler/tests/driver.rs`） |
| [`extern`: transportable type](../spec/extern.md#transportable-type) | P/N/E: `validates_extern_signatures_recursively`（`compiler/tests/check.rs`） | aggregate and opaque ABI tests（`compiler/tests/driver/artifacts.rs`） |
| [`extern`: host contractと安全性](../spec/extern.md#host-contract) | trusted host側の規範であり、mal compilerのadmission対象外 | generated headerを使用する全host fixture |
| [`extern`: boundary transport](../spec/extern.md#boundary-transport) | P/E: Symbol型・signature検査（`compiler/tests/check.rs`） | copy、host mutation、allocation/length failure tests（`compiler/tests/driver/artifacts.rs`）、`socket-packet` example（`compiler/tests/driver.rs`） |
| [`extern`: ABIとadapter](../spec/extern.md#abi-と-adapter) | P/N: generated declaration検査（`compiler/tests/driver/artifacts.rs`） | checked-in host adapter、`socket-packet`のgenerated macro使用（`compiler/tests/driver.rs`） |
| [`c-host-abi`: build model](../spec/c-host-abi.md#build-model) | P/N: buildとtoolchain failure tests（`compiler/tests/driver.rs`） | 複数host inputと全checked-in example（`compiler/tests/driver.rs`） |
| [`c-host-abi`: generated headerとsymbol](../spec/c-host-abi.md#generated-header) | P/E: `extracts_the_host_interface_without_lowering_value_bindings`（`compiler/tests/core.rs`）、header assertion群（`compiler/tests/driver/artifacts.rs`） | headerをincludeするhost fixture群 |
| [`c-host-abi`: Host value mapping](../spec/c-host-abi.md#host-value-mapping) | P/E: scalar/aggregate/opaque/Symbol header tests（`compiler/tests/driver/artifacts.rs`） | 各ABI round-trip test、`bridges_symbol_parameters_and_results_through_the_public_c_abi`（`compiler/tests/driver/artifacts.rs`） |
| [`c-host-abi`: closure exclusion](../spec/c-host-abi.md#closure-exclusion) | N/E: `validates_extern_signatures_recursively`（`compiler/tests/check.rs`） | — |
| [`c-host-abi`: failure](../spec/c-host-abi.md#failure) | P: sum resultと`mal_trap` declaration tests（`compiler/tests/driver/artifacts.rs`） | `mal_trap`を含むnative trap tests、`socket-packet`のrecoverable payload rejection |

## Programと字句・文法

| 規範 | P / N / E | X |
|---|---|---|
| [`programs`: programとsource file](../spec/programs.md#program-と-source-file) | P/N/E: require parse、visibility、衝突、cycle、path tests（`compiler/tests/parser.rs`、`compiler/tests/resolve.rs`、`compiler/tests/driver.rs`） | `builds_public_functions_from_required_files_with_private_helpers`、`mini-database` example（`compiler/tests/driver.rs`） |
| [`programs`: top-level item](../spec/programs.md#top-level-item) | P/N/E: `type_and_external_declarations_are_visible_across_the_unit`（`compiler/tests/resolve.rs`）、`rejects_effectful_top_level_initializers`（`compiler/tests/check.rs`） | `examples/`のchecked-in program（`compiler/tests/driver.rs`） |
| [`programs`: entry point](../spec/programs.md#entry-point) | P/N/E: `builds_a_constant_main_through_the_llvm_artifact_set`、`passes_process_arguments_through_the_llvm_entry_bridge`（`compiler/tests/driver/artifacts.rs`） | public `build` example tests（`compiler/tests/driver.rs`） |
| [`grammar`: sourceとidentifier](../spec/grammar.md#source-と-identifier) | P/N/E: keyword、whitespace/comment、identifier、UTF-8 diagnostic tests（`compiler/tests/lexer.rs`、`compiler/src/source.rs`） | frontendを通る全native test |
| [`grammar`: numeric separator](../spec/grammar.md#numeric-separator) | P/N/E: numeric separator tests（`compiler/tests/lexer.rs`） | `integer-and-byte`、`strict-float` examples（`compiler/tests/driver.rs`） |
| [`grammar`: 文法概要](../spec/grammar.md#文法概要) | P/N/E: parser suite（`compiler/tests/parser.rs`）、`rejects_the_unsupported_fat_arrow_token`（`compiler/tests/lexer.rs`） | 全checked-in example（`compiler/tests/driver.rs`） |
| [`grammar`: operator precedence](../spec/grammar.md#operator-precedence) | P/N/E: precedence、call binding、non-associative rejection（`compiler/tests/parser.rs`） | integer/float/Bool operator native tests（`compiler/tests/driver/artifacts.rs`） |
| [`grammar`: 存在しない構文](../spec/grammar.md#存在しない構文) | N: unknown token/nameとsyntax rejection（`compiler/tests/lexer.rs`、`compiler/tests/parser.rs`、`compiler/tests/resolve.rs`） | — |

## Public path

`compiler/tests/driver.rs`はpublic `malc` executableを通してhelp、version、usage error、`check`、`emit-header`、`emit-host`、
`build`、filesystem error、C compiler起動失敗、C compiler non-zero exit、および`examples/`のchecked-in programを
検証する。これにより上表のstage-focused testが利用者向け経路にも接続されていることを確認する。
