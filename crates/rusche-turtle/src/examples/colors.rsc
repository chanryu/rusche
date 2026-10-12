; Color wheel of squares using a stepped RGB palette
(define (palette i)
  (list
    (% (* i 37) 256)
    (% (* i 67) 256)
    (% (* i 97) 256)))

(pensize 2)
(hideturtle)
(define i 0)
(while (< i 36)
  (define c (palette i))
  (pencolor (car c) (cadr c) (caddr c))
  (repeat 4 (forward 90) (right 90))
  (right 10)
  (set! i (+ i 1)))
