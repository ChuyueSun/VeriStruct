from pathlib import Path
from typing import Dict, List
import re

class LemmaPreprocessor:
    """
    Module for preprocessing Verus code by inserting lemmas from a specified directory
    before passing the code to the planner.
    """

    def __init__(self, lemmas_dir: str, logger):
        """Initialize the preprocessor with the lemmas directory."""
        self.lemmas_dir = lemmas_dir
        self.logger = logger
        self.lemmas = {}

    def load_lemmas(self, target_code: str = None) -> Dict[str, str]:
        """Load lemma files from the specified directory.
        If target_code is provided, only load lemmas relevant to that code.
        """
        lemmas = {}
        lemma_path = Path(self.lemmas_dir)
        
        if not lemma_path.exists():
            self.logger.warning(f"Lemmas directory {self.lemmas_dir} does not exist")
            return lemmas
            
        # Define keywords to look for
        keywords = ["saturating_sub"]  # Add more keywords as needed
        
        # If target code is provided, filter keywords to only those present in the code
        if target_code is not None:
            keywords = [k for k in keywords if k in target_code]
            if not keywords:
                self.logger.debug("No relevant keywords found in target code")
                return lemmas
        
        for file in lemma_path.glob("*.rs"):
            try:
                with open(file, 'r') as f:
                    content = f.read()
                    # Only load the lemma if it contains any of the needed keywords
                    matching_keywords = [k for k in keywords if k in content]
                    if matching_keywords:
                        lemmas[file.stem] = content
                        self.logger.info(f"Loaded lemma from {file.name} (matched keywords: {matching_keywords})")
                    else:
                        self.logger.debug(f"Skipped lemma {file.name} (no keyword matches)")
            except Exception as e:
                self.logger.error(f"Error loading lemma {file}: {str(e)}")
        
        self.lemmas = lemmas
        return lemmas

    def process_code(self, code: str) -> str:
        """Process the code by inserting lemmas after verus!{ marker."""
        if not self.lemmas:
            self.logger.warning("No lemmas loaded, returning original code")
            return code
        
        # Find the position of verus!{ with any spaces between ! and {
        match = re.search(r'verus!\s*{', code)
        if not match:
            self.logger.warning("No 'verus!{' marker found in code")
            return code
        
        # Insert position is right after the marker
        insert_pos = match.end()
        
        # Combine all lemmas with newlines
        lemma_text = "\n".join(self.lemmas.values())
        
        # Insert lemmas after the marker
        new_content = code[:insert_pos] + "\n" + lemma_text + code[insert_pos:]
        self.logger.info(f"Successfully inserted {len(self.lemmas)} lemmas")
        
        return new_content

    def preprocess(self, code: str) -> str:
        """Main preprocessing function that loads lemmas and processes the code.
        Only inserts lemmas that are relevant to the code being processed."""
        # Load lemmas that are relevant to this code
        self.load_lemmas(target_code=code)
        return self.process_code(code)