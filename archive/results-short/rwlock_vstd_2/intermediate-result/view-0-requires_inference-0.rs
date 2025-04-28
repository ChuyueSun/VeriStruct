


// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: `main` function not found in crate `tmpo39mhoga`
// {"$message_type":"diagnostic","message":"`main` function not found in crate `tmpo39mhoga`","code":{"code":"E0601","explanation":"No `main` function was found in a binary crate.\n\nTo fix this error, add a `main` function:\n\n```\nfn main() {\n    // Your program will start here.\n    println!(\"Hello world!\");\n}\n```\n\nIf you don't know the basics of Rust, you can look at the\n[Rust Book][rust-book] to get started.\n\n[rust-book]: https://doc.rust-lang.org/book/\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo39mhoga","byte_start":2,"byte_end":2,"line_start":2,"line_end":2,"column_start":2,"column_end":2,"is_primary":true,"text":[{"text":"","highlight_start":2,"highlight_end":2}],"label":"consider adding a `main` function to `/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo39mhoga`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0601]: `main` function not found in crate `tmpo39mhoga`\n --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo39mhoga:2:2\n  |\n2 |\n  | ^ consider adding a `main` function to `/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo39mhoga`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0601`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0601`.\n"}
//
//
