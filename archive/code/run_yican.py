import subprocess, os
from multiprocessing import Pool

def gen_task(task_name: str):
    ans = []
    for j in range(0, 3):
        os.system(f'mkdir ../results/{task_name}_{j}')
        tmp = f'python3 main.py --input ../verus_lang_benchmarks/yican-revised/{task_name}_todo.rs --output ../results/{task_name}_{j}/result.rs --config config-yican.json > ../results/{task_name}_{j}/log.txt 2>&1'
        ans.append(tmp)
    return ans

short_tasks = [
    'option',
    'agreement_type_inv',
    'agreement',
    'even_cell',
    'invariants',
    'rwlock_vstd',
    'rfmig_script'
]

large_tasks = [
    'set_from_vec',
    'rb_type_invariant',
    'oneshot',
    'monotonic_counter',
    'log',
    'frac',
    'doubly_linked',
    'doubly_linked_xor',
    'basic_lock2',
    'basic_lock1',
]

os.system('rm -rf ../results/*')
all_tasks = []
for bench in ['rb_type_invariant']:
    all_tasks += gen_task(bench)

for task in all_tasks:
    print(task, flush=True)
    subprocess.run(task, shell=True)
