"""
Template utilities for loading and processing prompts.
"""

import os
import re
from pathlib import Path
from string import Template
from typing import Dict, List, Optional

from configs.sconfig import config

"""
Automatically collects all templates in the prompts directory.
This is a dictionary of template names to template strings.
"""
templates: dict[str, Template] = {}

# Use relative path for prompts directory
current_dir = Path(__file__).parent
prompts_dir = current_dir  # Since template.py is already in the prompts directory

try:
    for mdfile in os.listdir(prompts_dir):
        if mdfile.endswith(".md"):
            with open(os.path.join(prompts_dir, mdfile), "r") as f:
                templates[mdfile[:-3]] = Template(f.read())
except FileNotFoundError:
    print(f"Warning: Could not find prompts directory at {prompts_dir}")
    print("Creating empty templates dict")
    templates = {}


def fill_template(name: str, keys: dict):
    """
    Generates a prompt from a template and a dictionary of keys.
    :param name: The name of the template to use.
    :param keys: A dictionary of keys to use in the template.
    :return: The generated prompt.
    
    Example:
        fill_template("plan_system", {"modules": "42", "task_overview": "42"})
        This will return the prompt with the keys replaced with the values, from
        "plan_system.md".
        
    Note:
        This function supports both ${var} and {{var}} style template variables.
    """
    if name not in templates:
        print(f"Warning: Template {name} not found. Using empty template.")
        return ""
    
    # Add default values
    keys["_blank"] = ""
    keys["_blanks"] = ""
    
    # First use string.Template to handle ${var} style variables
    template_content = templates[name].substitute(keys)
    
    # Then handle {{var}} style variables
    for key, value in keys.items():
        template_content = template_content.replace(f"{{{{{key}}}}}", str(value))
    
    return template_content


# New functions for loading Verus prompts directly


def load_prompt(filename: str) -> str:
    """
    Load a prompt from a markdown file in the prompts directory.

    Args:
        filename: Name of the markdown file (with or without .md extension)

    Returns:
        The content of the prompt file as a string
    """
    if not filename.endswith(".md"):
        filename = f"{filename}.md"

    # Get the directory of this file
    current_dir = Path(os.path.dirname(os.path.abspath(__file__)))
    file_path = current_dir / filename

    if not file_path.exists():
        raise FileNotFoundError(f"Prompt file not found: {file_path}")

    return file_path.read_text()


def add_seq_knowledge_if_needed(code: str, instruction: str) -> str:
    """
    Add Seq knowledge to the instruction if the code contains Seq references.

    Args:
        code: The Verus code to check for Seq usage
        instruction: The current instruction

    Returns:
        Updated instruction with sequence knowledge if needed
    """
    if "Seq" in code:
        instruction += f"\n\n{load_prompt('verus_seq')}"
    return instruction


def build_instruction(
    base_instruction: str,
    add_common: bool = True,
    add_view: bool = False,
    add_invariant: bool = False,
    add_requires_ensures: bool = False,
    code: Optional[str] = None,
) -> str:
    """
    Build a complete instruction by combining various prompt components.

    Args:
        base_instruction: The main instruction for the specific module
        add_common: Whether to add common Verus knowledge
        add_view: Whether to add View refinement guidelines
        add_invariant: Whether to add invariant guidelines
        add_requires_ensures: Whether to add requires/ensures formatting info
        code: The Verus code to analyze for Seq usage

    Returns:
        Complete instruction with all requested components
    """
    instruction = base_instruction

    if add_common:
        instruction += f"\n\n{load_prompt('verus_common')}"

    if add_view:
        instruction += f"\n\n{load_prompt('verus_view')}"

    if add_invariant:
        instruction += f"\n\n{load_prompt('verus_invariant')}"

    if add_requires_ensures:
        instruction += f"\n\n{load_prompt('verus_requires_ensures')}"

    # Add Seq knowledge if needed and code is provided
    if code:
        instruction = add_seq_knowledge_if_needed(code, instruction)

    return instruction
