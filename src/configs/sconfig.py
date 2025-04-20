import json, os

"""
Automatically collect all json files in the configs directory and load them into a dictionary.
"""
configs = {}
for jfile in os.listdir('configs'):
    if jfile.endswith('.json'):
        with open(os.path.join('configs', jfile), 'r') as f:
            try:
                configs[jfile[:-5]] = json.load(f)
            except json.JSONDecodeError as e:
                assert False, f"Error Loading Config {jfile}: {e}"

config = configs['config-yican'] # Default Set to Azure
def reset_config(name: str):
    """
    Reset the config to the specified name.
    :param name: The name of the config to reset to.
    """
    global config
    if name in configs:
        config = configs[name]
    else:
        assert False, (f"Config {name} not found.")