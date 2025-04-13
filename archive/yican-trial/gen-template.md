Verus is a tool for verifying the correctness of code written in Rust. Below, we provide some backgrounds on verus.
## Verus Background
### Introduction to Verus Language
Verus is an extension of Rust designed for formal verification of low-level systems code. It enables developers to write specifications and proofs alongside executable code using a unified syntax based on Rust. Verus leverages classical logic, automated theorem provers (like SMT solvers), and Rust’s type system to help verify correctness properties of programs. Below, we explore key features of Verus with explanations and examples.
---
### **Verus Modes: Spec Function, Proof Function, and Execution Function**
Verus explicitly separates code into three modes: **spec**, **proof**, and **exec**, each serving distinct roles.
#### 1. **Spec Functions**
- **Purpose**: Used to write mathematical specifications and logic that act as non-executable code. Spec functions exist purely in the verification context and describe properties without being part of the runtime.
- **Key Characteristics**:
  - Pure mathematical abstraction.
  - Non-executable (cannot read/write memory).
  - Used to define properties for verification.
  - Guaranteed to terminate.
- **Example**:
  ```rust
  spec fn max_spec(a: int, b: int) -> int {
      if a > b { a } else { b }
  }
  ```
#### 2. **Proof Functions**
- **Purpose**: Encapsulate proof code written to help the verifier confirm correctness. Proof functions are executed only during verification (compile-time) and are not included in runtime artifacts.
- **Key Characteristics**:
  - Can manipulate proof objects.
  - Serve as intermediate logical steps in proofs.
  - Not exported to runtime.
- **Example**:
  ```rust
  proof fn check_max_spec_correctness() {
      assert(max_spec(3, 5) == 5);
  }
  ```
#### 3. **Executable Functions**
- **Purpose**: Regular Rust functions for executable code that manipulate and modify runtime state.
- **Key Characteristics**:
  - Can interact with memory and computation.
  - Adhere to Rust's ownership and borrowing rules.
  - Ensures verifiable properties via requires/ensures.
- **Example**:
  ```rust
  fn max_exec(a: i32, b: i32) -> i32 {
      if a > b { a } else { b }
  }
  ```
#### **Comparison:**
Key differences are summarized below:
| **Feature**        | **Spec Functions**       | **Proof Functions**      | **Execution Functions**   |
|---------------------|--------------------------|--------------------------|---------------------------|
| Purpose             | Describe properties      | Intermediate logic/proofs| Runtime code              |
| Runtime Execution   | No                       | No                       | Yes                       |
| Reads/Writes Memory | No                       | No                       | Yes                       |
| Main Use Case       | Specification            | Verification             | Implementation            |
---
### **Code Modes: Spec Code, Proof Code, Executable Code**
Verus delineates code into **spec**, **proof**, and **exec**, ensuring clarity and preventing unintended mixing.
1. **Spec Code**:
   - Mathematical abstractions (e.g., integers `int` instead of 32-bit integers `i32`).
   - Used in specifications (`requires`, `ensures`).
   - Not compiled into an executable binary.
   - Example:
     ```rust
     spec fn factorial_spec(n: int) -> int {
         if n <= 0 { 1 } else { n * factorial_spec(n - 1) }
     }
     ```
2. **Proof Code**:
   - Represents auxiliary constructs for proving logical correctness.
   - Provides logical arguments for assertions.
   - Example:
     ```rust
     proof fn factorial_proof(n: int) {
         requires(n >= 0);
         ensures(factorial_spec(n) > 0);  // Logical proof that factorial is positive
         ...
     }
     ```
3. **Executable Code**:
   - Actual runtime logic written in Rust syntax.
   - Interfaces between verified code and unverified system code.
   - Example:
     ```rust
     fn factorial_exec(n: i32) -> i32 {
         let mut result = 1;
         for i in 1..=n {
             result *= i;
         }
         result
     }
     ```
#### **Comparison**:
| **Aspect**                | **Spec Code** | **Proof Code** | **Exec Code** |
|---------------------------|---------------|----------------|---------------|
| Compile Target            | Not compiled  | Not compiled   | Compiled      |
| Runtime Execution         | No            | No             | Yes           |
| Used For                  | Specifications| Proof verifications| Implementations |
| Guarantees                | Logical models| Soundness proofs| Practical correctness |
---
### **Contracts: Requires, Ensures, and Recommends**
1. **`requires`**:
   - Specifies **preconditions**.
   - Must hold true before a function is executed.
   - Used in both spec and exec contexts.
   - Example:
     ```rust
     fn divide(n: i32, d: i32) -> i32
         requires(d != 0)  // Denominator must not be zero
     {
         n / d
     }
     ```
2. **`ensures`**:
   - Specifies **postconditions**.
   - Must be true after a function completes.
   - Helps ensure correctness of return values.
   - Example:
     ```rust
     fn add_one(n: i32) -> i32
         ensures(result == n + 1)
     {
         n + 1
     }
     ```
3. **`recommends`**:
   - Indicates **advisable** preconditions.
   - Less strict than `requires`. Code can still operate if not satisfied (unlike `requires`).
   - Typically used to recommend better usage patterns.
   - Example:
     ```rust
     fn process_data(data: &[i32])
         recommends(data.len() > 0)  // Advises but does not mandate non-empty array
     {
         if data.is_empty() {
             // Handle edge cases
         }
     }
     ```
#### **`recommends` vs. `requires`:**
| **Feature**        | **`recommends`**             | **`requires`**           |
|---------------------|------------------------------|--------------------------|
| Strictness          | Advisory, not mandatory     | Mandatory for correctness |
| Behavior on Failure | Executable, though not ideal| Errors or undefined       |
| Usage               | Performance hints, optional | Contractual obligations  |
---
### **Other Key Features**
1. **Invariants**:
   - Properties that must hold true during specific program states.
   - Used in loops and data structures.
   - Example (loop invariant):
     ```rust
     fn sum_array(arr: &[i32]) -> i32 {
         let mut sum = 0;
         let mut i = 0;
         while i < arr.len()
             invariant(sum == arr[0..i].iter().sum())
         {
             sum += arr[i];
             i += 1;
         }
         sum
     }
     ```
2. **Type Invariant**:
   - Enforces correctness constraints at the type level.
   - Example:
     ```rust
     struct NonNegative {
         value: int,
     }
     impl NonNegative {
         fn new(x: int) -> Self
             requires(x >= 0)
         {
             NonNegative { value: x }
         }
     }
     ```
3. **`decreases`**:
   - Used in defining **termination metrics** for recursive functions.
   - Example:
     ```rust
     fn factorial(n: int) -> int
         decreases(n)
     {
         if n == 0 {
             1
         } else {
             n * factorial(n - 1)
         }
     }
     ```
   `decreases` helps the verifier ensure the recursion will terminate by guaranteeing that the argument strictly reduces with each recursive call.
---
### **Conclusion**
Verus provides a powerful framework for writing specifications, proofs, and executable Rust code. By separating concerns between spec, proof, and exec domains, and employing contracts such as `requires` and `ensures`, it ensures correctness in both logic and implementation. With constructs like invariants and `decreases`, Verus builds on traditional verification techniques to simplify verification and formal reasoning for system-level software. Its integration with Rust's syntax makes it approachable for developers familiar with Rust.Now, you have been a wonderful expert at Verus, and a good programmer that can write detailed comments to make the code more readable.
## Instruction
Below, there will be a verus code that does lacks the pre-post condition (See: `TODO: FILL THIS` in the code).
Please:
- carefully read the code
- understand what it is doing, and
- illustrate what properties that each assertion requires to prove.
- illustrate in natural language, item by item, in details, on how to set up properties so that all assertions can be proved
- add the pre-post condition.
### NOTE
- Do not modify the code, only add pre- and post-conditions.
- Wrap the commented code using ```rust and ```.
- Think over to guarantee the correctness of the result
## Verus Code To Be Verified
```rust