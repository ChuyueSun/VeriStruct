from typing import Dict, List, Optional, Any
import os
import json
from pathlib import Path
from prompts.template import fill_template

class BaseModule:
    """
    Base class for all modules.
    Each module should implement the `exec` method.
    Each module should also have a name and a description.
    The exec method:
    - takes Context as input
    - do something
    - output a string as the llm generated code
    """
    def __init__(self, name: str, desc: str, 
                 hdn=None, 
                 example=None, 
                 default_system=None, 
                 config_path: Optional[str] = None):
        self.name = name
        self.desc = desc
        self.hdn = hdn
        self.example = example
        self.default_system = default_system
        self.config = self._load_config(config_path) if config_path else {}
    
    def _load_config(self, config_path: str) -> Dict[str, Any]:
        """
        Load configuration from a JSON file.
        
        Args:
            config_path: Path to the JSON configuration file
            
        Returns:
            Dict containing configuration
        """
        try:
            with open(config_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading config from {config_path}: {e}")
            return {}
    
    def _load_prompt(self, prompt_path: str) -> str:
        """
        Load prompt from a markdown file in the prompts directory.
        
        Args:
            prompt_path: Path to the markdown prompt file
            
        Returns:
            String containing the prompt
        """
        try:
            prompt_file = Path("src/prompts") / prompt_path
            with open(prompt_file, 'r') as f:
                return f.read()
        except Exception as e:
            print(f"Error loading prompt from {prompt_path}: {e}")
            return ""
            
    def _fill_template(self, template_name: str, replacements: Dict[str, str]) -> str:
        """
        Fill a template with the given replacements.
        
        Args:
            template_name: Name of the template file (without extension)
            replacements: Dictionary of replacements for the template
            
        Returns:
            Filled template string
        """
        try:
            return fill_template(template_name, replacements)
        except Exception as e:
            print(f"Warning: Error filling template {template_name}: {e}")
            # If template filling fails, return a sensible default or empty string
            return ""
        
    def exec(self, context) -> str:
        """
        Execute the module with the given context.
        
        Args:
            context: Context object containing trial information
            
        Returns:
            String result of execution that will be used as new trial code
        """
        raise NotImplementedError("Subclasses must implement exec() method")