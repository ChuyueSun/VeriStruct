# Verus Agent Task Overview

The Verus Agent is designed to help create and fix formal verification proofs for Rust programs using the Verus verification framework.

## Verification Process

The verification process follows one of two strictly defined workflows:

### Workflow 1: Complete Sequence (for data structures with Views)
1. **View Inference**: Create a view function that maps concrete state to abstract state.
2. **View Refinement**: Refine and improve the view function to handle edge cases and ensure it's complete.
3. **Invariant Inference**: Create invariants that express the properties that must be maintained.
4. **Specification Inference**: Add requires/ensures clauses to functions that specify their behavior.

### Workflow 2: Direct Specification (for standalone functions)
1. **Specification Inference**: Add requires/ensures clauses to functions that specify their behavior.

After completing either workflow, if there are verification errors, the agent will attempt to repair them using various repair modules.

The agent will execute one of these workflows in the specified order based on the current state of the code. 