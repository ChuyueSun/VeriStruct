import subprocess
from multiprocessing import Pool

commands = [
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_16.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_17.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_18.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_19.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_20.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_21.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_22.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_23.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_24.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    "python main.py --input ../rb_type_invariant_todo.rs --output o1/rb_planner_25.rs --config config-azure.json --immutable-functions test_enqueue_dequeue_generic",
    
]

def run_command(cmd):
    return subprocess.run(cmd, shell=True, check=True)
if __name__ == "__main__":
    with Pool(processes=5) as p:  # 5 parallel processes
        p.map(run_command, commands)
