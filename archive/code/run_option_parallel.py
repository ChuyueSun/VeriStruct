import subprocess
from multiprocessing import Pool

commands = [
    "python3 main.py --input ../verus_lang_benchmarks/option_todo.rs --output option_1.rs --config config-yican.json",
    "python main.py --input ../verus_lang_benchmarks/option_todo.rs --output option_2.rs --config config-azure.json",
    "python main.py --input ../verus_lang_benchmarks/option_todo.rs --output option_3.rs --config config-azure.json",
    "python main.py --input ../verus_lang_benchmarks/option_todo.rs --output option_4.rs --config config-azure.json",
    "python main.py --input ../verus_lang_benchmarks/option_todo.rs --output option_5.rs --config config-azure.json",
]


def run_command(cmd):
    return subprocess.run(cmd, shell=True)


if __name__ == "__main__":
    # Up to 5 parallel processes
    with Pool(processes=5) as pool:
        pool.map(run_command, commands[0:1])
