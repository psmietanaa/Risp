(
    (fn addNumbers (x y) (+ x y))
    (let x 1)
    (let y 2)
    (if (= (addNumbers x y) 3)
        (print Success)
        (print Failure)
    )
)