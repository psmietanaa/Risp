(
    (let x 1)
    (let y (+ 1 (* 1 1)))
    (fn addOne (x) (+ x 1))
    (let z (addOne y))
    (print x)
    (print y)
    (print z)
)