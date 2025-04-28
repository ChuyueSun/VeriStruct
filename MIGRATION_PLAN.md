# VerusAgent Migration Plan: @code → @src

## Overview
This plan outlines the migration of code from `/home/chuyue/VerusAgent/archive/code` (referred to as `@code`) to `/home/chuyue/VerusAgent/src` (referred to as `@src`), which represents a more modular and structured implementation.

## 1. Inventory & Core Functionality Analysis

| Original (@code) | Target (@src) | Key Functionality | Action Required |
|------------------|---------------|-------------------|----------------|
| utils.py | modules/utils.py | Utilities for code manipulation | Migrate missing functions |
| veval.py | modules/veval.py | Verus evaluation | Review for missing features |
| generation.py | modules/view_inference.py | View generation | Extract core logic |
| refinement.py | modules/view_refinement.py | Code refinement | Extract core logic |
| houdini.py | modules/houdini.py | Houdini algorithm | Review for parity |
| lynette.py | modules/lynette.py | Lynette integration | Review for parity |
| reader.py | doc/naive_reader.py | Documentation reader | Complete migration |
| main.py | main.py | Main entry point | Ensure CLI compatibility |
| infer.py | infer.py | LLM inference | Ensure feature parity |

## 2. Utilities Migration (Primary Focus)

Missing utility functions to migrate from `@code/utils.py` to `@src/modules/utils.py`:
- `AttrDict` (though this may be in src/configs/sconfig.py already)
- `remove_comment` and `remove_rust_comments`
- `get_nonlinear_lines`
- `code_change_is_safe`
- `get_func_body`
- `check_changed_code_v2`
- `evaluate` (to be renamed or merged with existing evaluation functions)
- `compress_nl_assertion`
- `remove_redundant_loopinv`
- `same_code_verus`
- `insert_loop_isolation`
- `insert_lemma_func`
- `insert_proof_func`
- JSON utilities like `load_jsonl` and `dump_jsonl`

## 3. Module Refactoring

1. Generation Module:
   - Extract main functionality from @code/generation.py
   - Create class-based implementation in @src/modules/
   - Ensure compatibility with Context and Module system

2. Refinement Module:
   - Extract main functionality from @code/refinement.py
   - Create class-based implementation in @src/modules/ (potentially multiple smaller modules)
   - Ensure compatibility with Context and Module system

3. Reader Module:
   - Complete migration of @code/reader.py to @src/doc/naive_reader.py

## 4. Configuration Unification

1. Review all config files in @code/*.json and @src/configs/*.json
2. Standardize on one format
3. Update references throughout the codebase

## 5. Test Migration

1. Move @code/test_code/* to @src/tests/
2. Update imports and paths
3. Ensure tests can run with pytest

## 6. CLI & Entry Point

1. Ensure src/main.py incorporates all functionality from @code/main.py
2. Review and migrate shell scripts as needed

## 7. Parallel Script Migration

1. Create a unified parallel execution framework in @src
2. Replace scripts in @code/parallel_scripts/

## 8. Documentation Update

1. Update README_modules.md
2. Add migration guide (this file)
3. Document new module structure

## 9. Code Cleanup and Final Review

1. Run tests to ensure functionality
2. Remove duplicate or obsolete code
3. Final review of all migrated components

## Execution Strategy

We'll tackle this migration in the following order:
1. Utilities first - migrate core functions to establish foundation
2. Module refactoring - migrate high-level logic
3. Test migration - ensure correctness
4. Configuration and CLI updates
5. Documentation and cleanup
