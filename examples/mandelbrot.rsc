#!/usr/bin/env rusche-cli
;; Plot the Mandelbrot set in ASCII.
;;
;;   cargo run -p rusche-cli -- examples/mandelbrot.rsc
;;   cargo run -p rusche-cli -- examples/mandelbrot.rsc 120 40 ; custom WIDTH HEIGHT
;;
;; Rendering is CPU-bound; add `--release` for larger sizes.

;; Characters from "escapes quickly" to "stays bounded" (the set itself).
(define palette " .:-=+*#%@")

;; Region of the complex plane to render.
(define re-min -2.1)
(define re-max 0.7)
(define im-min -1.2)
(define im-max 1.2)

(define max-iter 30)

;; Optional WIDTH and HEIGHT command-line arguments.
;; (command-line) is (program script-path args...), so skip the first two.
(define args (cddr (command-line)))
(define width (if (null? args) 72 (str->num (car args))))
(define height (if (or (null? args) (null? (cdr args))) 24 (str->num (cadr args))))

;; Rusche has no `floor`; for x >= 0, x - (x mod 1) is the integer part.
(defun floor (x) (- x (% x 1)))

;; Count iterations of z <- z^2 + c (starting at z = 0) until |z| > 2,
;; or return max-iter if the point stays bounded. Tail-recursive.
(defun escape-count (cr ci)
  (defun iter (zr zi n)
    (let ((zr2 (* zr zr))
          (zi2 (* zi zi)))
      (cond ((= n max-iter) n)
            ((> (+ zr2 zi2) 4) n)
            (else (iter (+ (- zr2 zi2) cr)
                        (+ (* 2 zr zi) ci)
                        (+ n 1))))))
  (iter 0 0 0))

;; Map an escape count onto a palette character. Points near the boundary
;; escape slowly, so every two iterations advance one shade and anything
;; slower than that (including the set itself) gets the last character.
(defun shade (n)
  (let ((index (min (- (str-length palette) 1) (floor (/ n 2)))))
    (str-slice palette index (+ index 1))))

(define dx (/ (- re-max re-min) width))
(define dy (/ (- im-max im-min) height))

(let ((row 0))
  (while (< row height)
    (let ((ci (+ im-min (* row dy)))
          (col 0)
          (line ""))
      (while (< col width)
        (let ((cr (+ re-min (* col dx))))
          (set! line (str-append line (shade (escape-count cr ci)))))
        (set! col (+ col 1)))
      (display line)
      (newline))
    (set! row (+ row 1))))
