## Verus Specification Synthesis Task

Below, we introduce the work-in-progress verus specification synthesis task.

### Input

The input of this task consists of five parts:

1. **Verus Code:** The verus code that is not fully verified.
2. **Compilation Error:** The compiler error reported from the verus code above.
3. **Knowledge**: The knowldge of the components mentioned in the code.
4. **Examples**: Relevant examples that the agent may refer to.
5. **Failures:** The previous failures that the LLM should avoid to do again

### Output

The output of this task consists of a verus code that can be fully verified.