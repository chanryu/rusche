; Koch snowflake — the turtle walks a recursive fractal path.
(define (koch length depth)
  (if (<= depth 0)
      (forward length)
      (begin
        (koch (/ length 3) (- depth 1))
        (left 60)
        (koch (/ length 3) (- depth 1))
        (right 120)
        (koch (/ length 3) (- depth 1))
        (left 60)
        (koch (/ length 3) (- depth 1)))))

(define (snowflake length depth)
  (repeat 3
    (koch length depth)
    (right 120)))

(pencolor "blue")
(pensize 1.5)
(hideturtle)
(penup)
(goto -160 -100)
(setheading 0)
(pendown)
(snowflake 320 4)
