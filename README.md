# RustLisp

This is a simple Lisp interpreter written in Rust. It is based on the famous article by Peter Norvig called [(How to Write a (Lisp) Interpreter (in Python))](https://norvig.com/lispy.html).
It contains modules that cover lexing, parsing, and evaluating using. You can use RustLisp in an interactive mode (REPL) or by passing a lisp file as an argument! See [programs](programs/) folder for some examples.

# Language Overview
- [x] Define variables via the syntax ```(let my-var <Expr>)```.
- [x] Define functions via the syntax ```(fn my-fun (arg1 arg2 arg3) <Expr>)``` where the final ```<Expr>``` the function body.
- [x] If expressions of the form ```(if (<Expr>) (<Expr>) (<Expr>))``` where the first ```<Expr>``` is the if-predicate, the second ```<Expr>``` is the then-body, and the final ```<Expr>``` is the else-body.
- [x] Arithmetic operations ```+```, ```-```, ```*```, ```/```.
- [x] Boolean operations ```or```, ```and```, ```not```.
- [x] Equality comparison operators ```=```, ```!=```.
- [x] Print function that prints a pretty-formatted output of its input.

# Example Programs
- **Program 1**
    - Input
    ```
    (
        (let x 1)
        (let y (+ 1 (* 1 1)))
        (fn addOne (x) (+ x 1))
        (let z (addOne y))
        (print x)
        (print y)
        (print z)
    )
    ```
    - Output
    ```
    1
    2
    3
    ```
- **Program 2**
    - Input
    ```
    (
        (fn addNumbers (x y) (+ x y))
        (let x 1)
        (let y 2)
        (if (= (addNumbers x y) 3)
            (print Success)
            (print Failure)
        )
    )
    ```
    - Output
    ```
    Success
    ```
- **Program 3**
    - Input
    ```
    (
        (let x True)
        (if (x)
            (print Success-1)
            (print Failure-2)
        )
        (if (not False)
            (print Success-2)
            (print Failure-2)
        )
        (if (or False False (not False))
            (print Success-3)
            (print Failure-3)
        )
        (if (and True True (not True))
            (print Failure-4)
            (print Success-4)
        )
    )
    ```
    - Output
    ```
    Success-1
    Success-2
    Success-3
    Success-4
    ```

# Usage
- To **build** the program, use the command ```cargo build```
- To **run** the program, use the command ```cargo run```
- To **test** the program, use the command ```cargo test```
- To **clean** the program, use the command ```cargo clean```
