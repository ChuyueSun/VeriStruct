import os
from string import Template
from configs.sconfig import config
from pathlib import Path

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
        fill_template("plan_system", {"modules": "42", "task_desc": "42"})
        This will return the prompt with the keys replaced with the values, from
        "plan_system.md".
    """
    if name not in templates:
        print(f"Warning: Template {name} not found. Using empty template.")
        return ""
    keys['_blank'] = ''
    keys['_blanks'] = ''
    return templates[name].substitute(keys)