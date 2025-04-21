from typing import Dict, List, Optional, Any
import os
import json
from pathlib import Path
from prompts.template import fill_template
from infer import LLM

class BaseModule:
    """
    Base class for all modules.
    
    Attributes:
        hdn: Houdini algorithm implementation
        example: Examples for the module
        default_system: Default system prompt
    """
    
    def __init__(self, 
                 name: str, 
                 desc: str,
                 hdn=None, 
                 example=None, 
                 default_system=None, 
                 config_path: Optional[str] = None):
        """
        Initialize the BaseModule with optional components.
        
        Args:
            name: The name of the module
            desc: Description of what the module does
            hdn: Houdini algorithm implementation
            example: Examples for the module
            default_system: Default system prompt
            config_path: Path to config file
        """
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
        return fill_template(template_name, replacements)
        
    def exec(self, context) -> str:
        """
        Execute the module with the given context.
        
        Args:
            context: Context object containing trial information
            
        Returns:
            String result of execution that will be used as new trial code
        """
        raise NotImplementedError("Subclasses must implement exec() method")
