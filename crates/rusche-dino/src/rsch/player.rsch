;; Dino: run / jump / duck / fast-fall. Dead = freeze pose.

(define (on-ground?)
    (>= dino-y (- (ground-dino-y) 0.5)))

(define (step-player dt jump? duck?)
    ;; Rising edge so hold-jump does not auto-rejump.
    (let ((press-jump (and jump? (not prev-jump?))))
        (set! prev-jump? jump?)
        (if dead?
            ()
            (begin
                (when (and press-jump (on-ground?))
                    (set! started? true)
                    (set! dino-vy JUMP-V)
                    (set! ducking? false))
                (if (and (on-ground?) (>= dino-vy 0))
                    (begin
                        (set! ducking? (and duck? started?))
                        (set! dino-vy 0)
                        (set! dino-y (if ducking? (ground-duck-y) (ground-dino-y))))
                    (begin
                        ;; Duck while airborne = fast-fall.
                        (set! ducking? false)
                        (set! dino-vy (+ dino-vy (* GRAVITY dt (if duck? FAST-FALL 1))))
                        (set! dino-y (+ dino-y (* dino-vy dt)))
                        (when (and (>= dino-y (ground-dino-y)) (>= dino-vy 0))
                            (set! dino-y (ground-dino-y))
                            (set! dino-vy 0))))
                (when started?
                    (set! anim-t (+ anim-t dt)))))))

(define (dino-sprite-name)
    (cond (dead? 'dino-dead)
          ((not (on-ground?)) 'dino-jump)
          (ducking? (if (< (% (* anim-t 10) 2) 1) 'dino-duck-0 'dino-duck-1))
          (else (if (< (% (* anim-t 10) 2) 1) 'dino-run-0 'dino-run-1))))

;; Slightly inset vs sprite bounds.
(define (dino-hitbox)
    (if ducking?
        (list (+ DINO-X 4) (+ dino-y 2) (- DINO-DUCK-W 8) (- DINO-DUCK-H 4))
        (list (+ DINO-X 4) (+ dino-y 4) (- DINO-W 8) (- DINO-H 8))))
