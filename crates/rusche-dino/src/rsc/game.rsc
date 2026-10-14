;; Frame entry point. Host calls (tick dt jump? duck?).

(define (tick dt jump? duck?)
    (when (and dead? jump? (not prev-jump?))
        (reset))
    (step-player dt jump? duck?)
    (step-field dt)
    (draw-scene))
