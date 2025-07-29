#!/usr/bin/env python3

import os
import sys
from pathlib import Path
from typing import List, Dict

def load_lemmas(lemmas_dir: str) -> Dict[str, str]:
    """Load all lemma files from the specified directory."""
    lemmas = {}
    lemma_path = Path(lemmas_dir)
    
    if not lemma_path.exists():
        print(f"Error: Lemmas directory {lemmas_dir} does not exist")
        sys.exit(1)
        
    for file in lemma_path.glob("*.rs"):
        with open(file, 'r') as f:
            content = f.read()
            lemmas[file.stem] = content
    
    return lemmas

def process_file(input_file: str, lemmas: Dict[str, str]) -> str:
    """Process a single file by inserting lemmas after verus!{ marker."""
    with open(input_file, 'r') as f:
        content = f.read()
    
    # Find the position of verus!{
    marker = "verus!{"
    marker_pos = content.find(marker)
    
    if marker_pos == -1:
        print(f"Warning: No '{marker}' found in {input_file}")
        return content
    
    # Insert position is right after the marker
    insert_pos = marker_pos + len(marker)
    
    # Combine all lemmas with newlines
    lemma_text = "\n".join(lemmas.values())
    
    # Insert lemmas after the marker
    new_content = content[:insert_pos] + "\n" + lemma_text + content[insert_pos:]
    return new_content

def main():
    if len(sys.argv) < 3:
        print("Usage: python lemma_processor.py <lemmas_dir> <input_file1> [input_file2 ...]")
        sys.exit(1)
    
    lemmas_dir = sys.argv[1]
    input_files = sys.argv[2:]
    
    # Load all lemmas
    lemmas = load_lemmas(lemmas_dir)
    if not lemmas:
        print("No lemma files found")
        sys.exit(1)
    
    # Process each input file
    for input_file in input_files:
        if not os.path.exists(input_file):
            print(f"Error: Input file {input_file} does not exist")
            continue
            
        try:
            new_content = process_file(input_file, lemmas)
            
            # Write the processed content back to the file
            with open(input_file, 'w') as f:
                f.write(new_content)
            print(f"Successfully processed {input_file}")
            
        except Exception as e:
            print(f"Error processing {input_file}: {str(e)}")

if __name__ == "__main__":
    main() 