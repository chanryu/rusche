#!/usr/bin/env rusche-cli
;; Records, destructuring, named let, case, and math (sqrt / expt).
;;
;;   cargo run -p rusche-cli -- examples/points.rsc

(define-record point (x y))

(defun point-dist (a b)
    (let (((ax ay) (list (point-x a) (point-y a)))
          ((bx by) (list (point-x b) (point-y b))))
        (sqrt (+ (expt (- bx ax) 2) (expt (- by ay) 2)))))

(defun classify (p)
    (case (round (point-dist (make-point 0 0) p))
        ((0) 'origin)
        ((1 2 3 4 5) 'near)
        (else 'far)))

(define samples
    (list (make-point 0 0)
          (make-point 3 4)
          (make-point 6 8)
          (make-point 10 0)))

(let loop ((pts samples))
    (when (not (null? pts))
        (let ((p (car pts)))
            (display "point (" (point-x p) ", " (point-y p) ")"
                     " dist=" (point-dist (make-point 0 0) p)
                     " -> " (classify p))
            (newline)
            (loop (cdr pts)))))

(unless (point? 42)
    (display "42 is not a point") (newline))
