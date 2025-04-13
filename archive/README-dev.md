# -verusyth

alias verus=/home/chuyue/verus/source/target-verus/release/verus

- Goal: generate verified ring buffer

  
  verified ring buffer = spec + code

### Step 0: Given executable rust code for ring buffer

### Step 1: Generate View for spec (success)
- can be formally verified with rust code: no
- termination condition: no verus errors relating to View

### Step 2: Generate requires/ensures for methods (todo)
- can be formally verified with rust code: no
- termination condition: no verus errors relating to pre/post conditions

### Step 3: Generate the rest of full proof (todo)
- can be formally verified with rust code: yes
- termination condition: Verus verified
