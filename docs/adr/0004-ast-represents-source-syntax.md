# AST Represents Source Syntax

Echo's AST represents parsed source syntax and source-level structure. Parser support for PHP compatibility or Echo extensions should add truthful AST shapes for the source construct, such as explicit `elseif` clauses, dynamic calls, references, imports, namespaces, includes, and append syntax, rather than encoding that syntax as unrelated statements or backend operations.

Semantic facts, type facts, symbol resolution, executable meaning, and runtime behavior belong in shared later layers: `echo_semantics`, HIR, MIR, `echo_codegen`, and `echo_runtime`. The trade-off is that the AST grows with the language surface, but downstream tools get a reliable source model and parser shortcuts do not become hidden semantic commitments.
