#!/usr/bin/env rusche-cli
;; Carve a perfect maze by recursive backtracking and print it in ASCII.
;;
;;   cargo run -p rusche-cli -- examples/maze.rsc
;;   cargo run -p rusche-cli -- examples/maze.rsc 24 12 ; rooms across, rooms down
;;
;; Each run is different: `random` is not seeded.

(load "lib/io.rsc")

;; (command-line) is (program script-path args...), so skip the first two.
(define args (cddr (command-line)))
(define cols (if (null? args) 20 (str->num (car args))))
(define rows (if (or (null? args) (null? (cdr args))) 10 (str->num (cadr args))))

;; One character per wall or passage. Rooms sit on the odd coordinates.
(define width (+ (* cols 2) 1))
(define height (+ (* rows 2) 1))

(define grid (vec))
(let ((i 0)
      (n (* width height)))
    (while (< i n)
        (vec-push grid "#")
        (set! i (+ i 1))))

(define (at x y)
    (+ (* y width) x))

(define (cell x y ch)
    (vec-set! grid (at x y) ch))

(define (room-x c) (+ (* c 2) 1))
(define (room-y r) (+ (* r 2) 1))

(define seen (vec))
(let ((i 0)
      (n (* cols rows)))
    (while (< i n)
        (vec-push seen 0)
        (set! i (+ i 1))))

(define (room-index c r)
    (+ (* r cols) c))

(define (seen? c r)
    (= (vec-get seen (room-index c r)) 1))

(define (mark c r)
    (vec-set! seen (room-index c r) 1)
    (cell (room-x c) (room-y r) " "))

;; 0 east, 1 south, 2 west, 3 north.
(define (dx d)
    (cond ((= d 0) 1)
          ((= d 2) -1)
          (else 0)))

(define (dy d)
    (cond ((= d 1) 1)
          ((= d 3) -1)
          (else 0)))

(define (in-bounds? c r)
    (and (>= c 0) (< c cols) (>= r 0) (< r rows)))

;; A random unvisited neighbor's direction, or false.
(define (pick-dir c r)
    (define order (vec 0 1 2 3))
    (let ((i 0))
        (while (< i 4)
            (let ((j (floor (* (random) 4)))
                  (a (vec-get order i)))
                (let ((b (vec-get order j)))
                    (vec-set! order i b)
                    (vec-set! order j a)))
            (set! i (+ i 1))))
    (define (scan i)
        (if (= i 4)
            false
            (let ((d (vec-get order i)))
                (let ((nc (+ c (dx d)))
                      (nr (+ r (dy d))))
                    (if (and (in-bounds? nc nr) (not (seen? nc nr)))
                        d
                        (scan (+ i 1)))))))
    (scan 0))

(define (open-wall c r d)
    (cell (+ (room-x c) (dx d))
          (+ (room-y r) (dy d))
          " "))

(define stack (list (list 0 0)))
(mark 0 0)

(while (not (null? stack))
    (let ((here (car stack)))
        (let ((c (car here))
              (r (cadr here)))
            (let ((d (pick-dir c r)))
                (if (not (eq? d false))
                    (begin
                        (open-wall c r d)
                        (let ((nc (+ c (dx d)))
                              (nr (+ r (dy d))))
                            (mark nc nr)
                            (set! stack (cons (list nc nr) stack))))
                    (set! stack (cdr stack)))))))

;; Door in the outer wall: enter at the north-west room, leave at the south-east.
(cell 0 1 " ")
(cell (- width 1) (- height 2) " ")

(let ((y 0))
    (while (< y height)
        (let ((x 0)
              (line ""))
            (while (< x width)
                (set! line (str-append line (vec-get grid (at x y))))
                (set! x (+ x 1)))
            (writeln line))
        (set! y (+ y 1))))
