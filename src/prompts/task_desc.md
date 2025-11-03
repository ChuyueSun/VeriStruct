# Verus Specification Code Synthesis Task

This file describes a verus Specification Code Synthesis task, which consists of the following four separate parts.

1. **Verus Code:** The verus code that is not fully verified.
2. **Compilation Error:** The compiler error reported from the verus code above.
3. **Knowledge**: The knowledge of the components mentioned in the code.
4. **Failures:**: The previous failures that the LLM should avoid to do again, each failure consists of:
   - the verus code and,
   - the compilation error.

## Verus Code

${verus_code}${_blank}

## Compilation Error

${rustc_out}${_blank}

## Knowledge

${knowledge}${_blank}

## Failures

${failures}${_blank}
