"""
Utilities for handling lemmas in Verus code.
"""

import os


def insert_proof_func(code: str, proof_func_dict: dict) -> str:
    """Insert the proof functions into the code.
    
    Args:
        code: The source code to insert into
        proof_func_dict: Dictionary mapping function names to their code
        
    Returns:
        Modified code with proof functions inserted after verus! macro
    """
    lines = code.splitlines()
    verus_line = -1
    for i, line in enumerate(lines):
        if "verus!" in line:
            verus_line = i
            break
    if verus_line == -1:
        return code
    proof_func_code = "\n\n".join(proof_func_dict.values())
    new_code = "\n".join(
        lines[: verus_line + 1] + [proof_func_code] + lines[verus_line + 1 :]
    )
    return new_code


def insert_lemma_func(code: str, lemma_names: list, lemma_path: str) -> str:
    """Insert existing already-proved lemmas into code.
    
    Args:
        code: The source code to insert into
        lemma_names: List of lemma names to insert
        lemma_path: Path to directory containing lemma files
        
    Returns:
        Modified code with lemmas inserted
    """
    for lemma_name in lemma_names:
        name = lemma_name
        if not name.endswith(".rs"):
            name = name + ".rs"
        input_file = os.path.join(lemma_path, name)
        lemma_code = open(input_file).read()
        lemma_func_dict = {lemma_name: lemma_code}
        code = insert_proof_func(code, lemma_func_dict)
    return code
