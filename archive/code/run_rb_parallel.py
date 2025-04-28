import subprocess
from multiprocessing import Pool

commands = [
    "python main.py --input ../rb_type_invariant_todo.rs --output rb_planner_6.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output rb_planner_7.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output rb_planner_8.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output rb_planner_9.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output rb_planner_10.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
]


def run_command(cmd):
    return subprocess.run(cmd, shell=True, check=True)


if __name__ == "__main__":
    with Pool(processes=5) as p:  # 5 parallel processes
        p.map(run_command, commands)
