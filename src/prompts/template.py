import os
from string import Template
from configs.sconfig import config

"""
Automatically collects all templates in the prompts directory.
This is a dictionary of template names to template strings.
"""
templates: dict[str, Template] = {}
prompts_dir = os.path.join(config['project_dir'], "prompts")
for mdfile in os.listdir(prompts_dir):
    if mdfile.endswith(".md"):
        with open(os.path.join(prompts_dir, mdfile), "r") as f:
            templates[mdfile[:-3]] = Template(f.read())


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
        raise ValueError(f"Template {name} not found.")
    keys['_blank'] = ''
    keys['_blanks'] = ''
    return templates[name].substitute(keys)