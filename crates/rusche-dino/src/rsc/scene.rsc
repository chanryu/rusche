;; Scene list for the host: clear / rect / sprite commands.

(define digit-names '(d0 d1 d2 d3 d4 d5 d6 d7 d8 d9))

(define (digit-name n)
    (nth digit-names (floor n)))

;; Five digits, leading zeros.
(define (pad5 n)
    (let ((n (floor (max 0 n))))
        (list (floor (% (/ n 10000) 10))
              (floor (% (/ n 1000) 10))
              (floor (% (/ n 100) 10))
              (floor (% (/ n 10) 10))
              (floor (% n 10)))))

(define (fg-rgb)
    (if (night?)
        (list NIGHT-FG-R NIGHT-FG-G NIGHT-FG-B)
        (list DAY-FG-R DAY-FG-G DAY-FG-B)))

(define (bg-rgb)
    (if (night?)
        (list NIGHT-BG-R NIGHT-BG-G NIGHT-BG-B)
        (list DAY-BG-R DAY-BG-G DAY-BG-B)))

(define (score-cmds x y value fg)
    (let ((digits (pad5 value))
          (r (car fg))
          (g (cadr fg))
          (b (caddr fg)))
        (list (list 'sprite (digit-name (car digits)) x y r g b)
              (list 'sprite (digit-name (cadr digits)) (+ x (+ DIGIT-W DIGIT-GAP)) y r g b)
              (list 'sprite (digit-name (caddr digits)) (+ x (* 2 (+ DIGIT-W DIGIT-GAP))) y r g b)
              (list 'sprite (digit-name (cadddr digits)) (+ x (* 3 (+ DIGIT-W DIGIT-GAP))) y r g b)
              (list 'sprite (digit-name (car (cddddr digits))) (+ x (* 4 (+ DIGIT-W DIGIT-GAP))) y r g b))))

(define (cddddr lst) (cdr (cddr (cdr lst))))

;; Score (blinks at milestones) + optional HI.
(define (hud-cmds fg)
    (let ((r (car fg))
          (g (cadr fg))
          (b (caddr fg))
          (score-x (- WIDTH 20 (* 5 (+ DIGIT-W DIGIT-GAP))))
          (hi-x (- WIDTH 20 (* 5 (+ DIGIT-W DIGIT-GAP)) 16 HI-W (* 5 (+ DIGIT-W DIGIT-GAP)))))
        (append
            (if (and (> blink-t 0) (< (% (* blink-t 8) 2) 1))
                ()
                (score-cmds score-x 10 score fg))
            (if (> hi-score 0)
                (append
                    (list (list 'sprite 'hi hi-x 10 r g b))
                    (score-cmds (+ hi-x HI-W 8) 10 hi-score fg))
                ()))))

(define (cloud-cmds fg)
    (let ((r (car fg))
          (g (cadr fg))
          (b (caddr fg)))
        (map (lambda (c)
                (list 'sprite 'cloud (car c) (cadr c) r g b))
             clouds)))

;; Horizon line + scrolling dirt ticks.
(define (ground-cmds fg)
    (let ((r (car fg))
          (g (cadr fg))
          (b (caddr fg)))
        (append
            (list (list 'rect 0 GROUND-Y WIDTH 2 r g b))
            (list (list 'rect (- 0 ground-scroll) (+ GROUND-Y 6) 4 2 r g b)
                  (list 'rect (- 40 ground-scroll) (+ GROUND-Y 8) 3 2 r g b)
                  (list 'rect (- 90 ground-scroll) (+ GROUND-Y 5) 5 2 r g b)
                  (list 'rect (- 140 ground-scroll) (+ GROUND-Y 7) 3 2 r g b)
                  (list 'rect (- 200 ground-scroll) (+ GROUND-Y 6) 4 2 r g b)
                  (list 'rect (- 260 ground-scroll) (+ GROUND-Y 8) 3 2 r g b)
                  (list 'rect (- 320 ground-scroll) (+ GROUND-Y 5) 5 2 r g b)
                  (list 'rect (- 380 ground-scroll) (+ GROUND-Y 7) 3 2 r g b)
                  (list 'rect (- 440 ground-scroll) (+ GROUND-Y 6) 4 2 r g b)
                  (list 'rect (- 500 ground-scroll) (+ GROUND-Y 8) 3 2 r g b)
                  (list 'rect (- 560 ground-scroll) (+ GROUND-Y 5) 5 2 r g b)
                  (list 'rect (- 620 ground-scroll) (+ GROUND-Y 7) 3 2 r g b)))))

;; 6 flaps/s from obs flap-t, not dino anim-t.
(define (bird-sprite-name obs)
    (if (= (% (floor (* (obs-flap obs) BIRD-FLAP-HZ)) 2) 0) 'bird-0 'bird-1))

;; size 2/3 → prebaked cluster sprites.
(define (cactus-sprite kind size)
    (cond ((eq? kind 'cactus-s)
           (cond ((= size 1) 'cactus-s)
                 ((= size 2) 'cactus-s2)
                 (else 'cactus-s3)))
          ((eq? kind 'cactus-l)
           (cond ((= size 1) 'cactus-l)
                 ((= size 2) 'cactus-l2)
                 (else 'cactus-l3)))
          (else 'cactus-s)))

(define (obs-sprite-name obs)
    (let ((kind (obs-kind obs)))
        (if (eq? kind 'bird)
            (bird-sprite-name obs)
            (cactus-sprite kind (obs-size obs)))))

(define (obstacle-cmds fg)
    (let ((r (car fg))
          (g (cadr fg))
          (b (caddr fg)))
        (map (lambda (obs)
                (list 'sprite (obs-sprite-name obs) (obs-x obs) (obs-y obs) r g b))
             obstacles)))

(define (dino-cmds fg)
    (let ((r (car fg))
          (g (cadr fg))
          (b (caddr fg)))
        (list (list 'sprite (dino-sprite-name) DINO-X dino-y r g b))))

(define (game-over-cmds fg)
    (if dead?
        (let ((r (car fg))
              (g (cadr fg))
              (b (caddr fg))
              (x (/ (- WIDTH GAME-OVER-W) 2))
              (y 55))
            (list (list 'sprite 'game-over x y r g b)))
        ()))

;; Back-to-front: clear → clouds → ground → obstacles → dino → HUD.
(define (draw-scene)
    (let ((fg (fg-rgb))
          (bg (bg-rgb)))
        (append (list (list 'clear (car bg) (cadr bg) (caddr bg)))
        (append (cloud-cmds fg)
        (append (ground-cmds fg)
        (append (obstacle-cmds fg)
        (append (dino-cmds fg)
        (append (hud-cmds fg)
                (game-over-cmds fg)))))))))
