# Lemma Preprocessor Module

## Overview

The Lemma Preprocessor injects helper lemmas into Verus source code before the planner runs. It scans the target code for known keywords and inserts the corresponding lemma implementations directly after the `verus!{` marker, ensuring that downstream modules have access to required proof utilities.

## Key Functions

### `load_lemmas`
Loads lemma files from a configured directory. Keywords are mapped to specific files and only lemmas whose keywords appear in the target code are read into memory.

### `process_code`
Inserts the loaded lemmas after the first `verus!{` marker in the code. If no lemmas are loaded or the marker is missing, the original code is returned unchanged.

### `preprocess`
High-level entry point that calls `load_lemmas` with the target code and then `process_code` to perform the insertion.

## Keyword-to-File Mapping
A built-in dictionary maps keywords to lemma filenames. For example:

```python
keyword_lemmas = {
    "saturating_sub": "mod.rs",  # Map saturating_sub to mod.rs which contains the lemma
    "bit": "bit.rs",              # Explicitly specify the lemma file to use
}
```
Only the files whose keywords appear in the code are loaded and inserted.

## Usage Example

```python
from logging import getLogger
from src.modules.lemma_preprocessor import LemmaPreprocessor

# Directory containing lemma files such as mod.rs and bit.rs
lemmas_dir = "lemmas"
pre = LemmaPreprocessor(lemmas_dir, getLogger(__name__))

code = """verus!{
    fn main() {
        // code using saturating_sub
    }
}"""

processed = pre.preprocess(code)
```
This configuration loads `lemmas/mod.rs` because the keyword `saturating_sub` appears in the input code. The lemma contents are inserted immediately after `verus!{` before further planning.
