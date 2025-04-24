"""
Registry for repair modules in VerusAgent.
Maps error types to appropriate repair modules.
"""

from typing import Dict, Tuple, Type, List, Optional, Any
import logging
from pathlib import Path
from modules.veval import VerusErrorType, VerusError
from modules.baserepair import BaseRepairModule

class RepairRegistry:
    """
    Registry for mapping error types to repair modules.
    This centralized mapping makes it easier to:
    1. Add new error types and repair modules
    2. Track which errors can be handled
    3. Select appropriate repair strategies
    """
    
    def __init__(self, config: Dict[str, Any], logger: logging.Logger, immutable_funcs: Optional[List[str]] = None):
        """
        Initialize the repair registry.
        
        Args:
            config: Configuration dictionary
            logger: Logger instance
            immutable_funcs: List of function names that should not be modified
        """
        self.config = config
        self.logger = logger
        self.immutable_funcs = immutable_funcs if immutable_funcs else []
        self.repair_modules = {}
        self.error_to_module_map = {}
        self.output_paths = {}
        
    def register_module(self, name: str, module: BaseRepairModule, 
                        error_types: List[VerusErrorType], output_path: str = None):
        """
        Register a repair module to handle specific error types.
        
        Args:
            name: Name of the repair module
            module: The repair module instance
            error_types: List of error types this module can handle
            output_path: Optional output path template for saving repair results
        """
        self.repair_modules[name] = module
        for error_type in error_types:
            self.error_to_module_map[error_type] = module
            if output_path:
                self.output_paths[error_type] = output_path
                
    def get_module_for_error(self, error: VerusError) -> Optional[BaseRepairModule]:
        """
        Get the appropriate repair module for a given error.
        
        Args:
            error: The Verus error to repair
            
        Returns:
            The repair module instance, or None if no module is registered
        """
        if error.error in self.error_to_module_map:
            return self.error_to_module_map[error.error]
        return None
    
    def get_output_path(self, error: VerusError) -> Optional[str]:
        """
        Get the output path for a given error type.
        
        Args:
            error: The Verus error
            
        Returns:
            The output path template, or None if not specified
        """
        if error.error in self.output_paths:
            return self.output_paths[error.error]
        return None
    
    def repair_error(self, context, error: VerusError, output_dir: Optional[Path] = None) -> Optional[str]:
        """
        Attempt to repair a specific error using the appropriate module.
        
        Args:
            context: The execution context
            error: The error to repair
            output_dir: Optional directory to save the repair result
            
        Returns:
            The repaired code if successful, None otherwise
        """
        module = self.get_module_for_error(error)
        if not module:
            self.logger.warning(f"No repair module registered for error type: {error.error.name}")
            return None
            
        self.logger.info(f"Attempting {error.error.name} repair with {module.name}...")
        result = module.exec(context, error)
        
        if output_dir and result:
            output_path = self.get_output_path(error)
            if output_path:
                output_file = output_dir / output_path
                output_file.write_text(result)
                self.logger.info(f"Saved {error.error.name} repair result to {output_file}")
        
        return result
    
    def repair_all(self, context, failures: List[VerusError], output_dir: Optional[Path] = None) -> Dict[VerusErrorType, str]:
        """
        Attempt to repair all errors in the list using appropriate modules.
        
        Args:
            context: The execution context
            failures: List of errors to repair
            output_dir: Optional directory to save repair results
            
        Returns:
            Dictionary mapping error types to repaired code
        """
        result_map = {}
        
        # Group failures by error type
        error_type_map = {}
        for failure in failures:
            if failure.error not in error_type_map:
                error_type_map[failure.error] = []
            error_type_map[failure.error].append(failure)
        
        # Process each error type
        for error_type, type_failures in error_type_map.items():
            if error_type in self.error_to_module_map:
                module = self.error_to_module_map[error_type]
                self.logger.info(f"Attempting {error_type.name} repair with {module.name}...")
                
                # Use the first failure of this type
                result = module.exec(context, type_failures[0])
                result_map[error_type] = result
                
                # Save the result if an output directory is provided
                if output_dir and result:
                    output_path = self.get_output_path(type_failures[0])
                    if output_path:
                        output_file = output_dir / output_path
                        output_file.write_text(result)
                        self.logger.info(f"Saved {error_type.name} repair result to {output_file}")
            else:
                self.logger.warning(f"No repair module registered for error type: {error_type.name}")
        
        return result_map 