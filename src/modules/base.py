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
    def __init__(self, name: str, desc: str):
        self.name = name
        self.desc = desc
    pass