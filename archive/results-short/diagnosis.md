### Even Cell

#### Main Reason

LLM does not know the API of `LocalInvariant`.

#### FIX procedure

1. Start from `even_cell_0/result.rs`.
2. prompt with compiler error report + definition of `LocalInvaraint`.
3. Manually remove `->()` in the code.
4. prompt with compiler error report.
5. o1 reports a correct result.

### rfmig_script

#### Reason

Need more fix rounds.