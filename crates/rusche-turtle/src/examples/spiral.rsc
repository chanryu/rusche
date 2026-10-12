; Growing square spiral
(pencolor "orange")
(pensize 2)
(define len 5)
(repeat 60
  (forward len)
  (right 91)
  (set! len (+ len 3)))
