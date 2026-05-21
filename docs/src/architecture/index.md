# Architecture

Internal design of ccalc: workspace layout, data flow, module responsibilities,
and design principles that guide every implementation decision.

## Contents

| Topic | What you will find |
|---|---|
| [Overview](./overview.md) | Workspace layout, data flow, dependency graph, design principles |
| [Engine Crate](./engine.md) | `ccalc-engine` public API, `Value` enum, `Env` type |
| [Parser](./parser.md) | Tokenizer, recursive-descent grammar, `Stmt` enum |
| [Evaluator](./evaluator.md) | AST evaluation, built-in dispatch, `exec_stmts` |
