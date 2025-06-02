import os
import glob
import subprocess
import sys

os.system('rm -rf results')
os.system('mkdir -p results')

for todo_path in glob.glob('benchmarks/*_todo.rs'):
    name = os.path.splitext(os.path.basename(todo_path))[0]
    test_file = f'benchmarks/{name}.rs'
    os.system(f'mkdir -p results/{name}')
    log_file = f'results/{name}/output.log'
    print(name, test_file, log_file)
    cmds = ['./run_agent.py', '--test-file', 
            test_file,
            '--output-dir', f'results/{name}',
            '>', log_file, '2>&1']
    try:
        subprocess.run(
            ' '.join(cmds),
            check=True,
            text=True,
            shell=True
        )
    except:
        print(f'Error running {name}, see {log_file} for details')
    