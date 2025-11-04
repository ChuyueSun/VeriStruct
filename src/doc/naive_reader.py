import os
import sys

if __name__ == "__main__":
    sys.path[0] = "/Users/syc/Desktop/-verusyth/src"
from src.configs.sconfig import config

# Lazy initialization to avoid accessing config at import time
_external_dir = None
_vstd_dir = None
vstd_base: dict[str, str] = {}


def _get_external_dir():
    """Get external directory lazily to avoid config access at import time."""
    global _external_dir
    if _external_dir is None:
        project_dir = config.get(
            "project_dir", os.path.dirname(os.path.dirname(os.path.dirname(__file__)))
        )
        _external_dir = os.path.join(project_dir, "external")
    return _external_dir


def _get_vstd_dir():
    """Get vstd directory lazily."""
    global _vstd_dir
    if _vstd_dir is None:
        _vstd_dir = os.path.join(_get_external_dir(), "vstd")
    return _vstd_dir


def search(path: str, path_id: str):
    if os.path.isdir(path):
        for file in os.listdir(path):
            if file.endswith(".rs"):
                cpath_id = path_id + "::" + file[:-3]
                cpath = os.path.join(path, file)
                vstd_base[cpath_id] = open(cpath).read()
            else:
                search(os.path.join(path, file), path_id + "::" + file)


def get_content(use_path: str):
    use_path = use_path.lstrip().rstrip()
    use_path = use_path.replace("*", "")
    use_path = use_path.replace("::", "/")
    use_path = use_path.replace(";", "")
    while use_path.endswith("/"):
        use_path = use_path[:-1]
    if use_path == "vstd" or use_path == "vstd/":
        return ""
    use_path = os.path.join(_get_external_dir(), use_path + ".rs")
    # print(use_path)
    if os.path.exists(use_path):
        return open(use_path, "r").read()
    else:
        return ""


def naive_retrieval(keyword: str, visible: set[str] = None):
    ans = ""
    for key, value in vstd_base.items():
        if visible is not None and key not in visible:
            continue
        if value.find(keyword) != -1:
            ans += f"#### {key}\n\n"
            ans += value

    return ans


if __name__ == "__main__":
    #    print(get_content('vstd::arithmetic::internals'))
    #    print(get_content('vstd::arithmetic::internals::*'))
    #    print(get_content('vstd::arithmetic::div_mod::*'))
    #    print(get_content('vstd::*'))
    #    print(get_content('vstd::invariant'))
    #    print(get_content('vstd::pcm_lib::*'))
    pass
