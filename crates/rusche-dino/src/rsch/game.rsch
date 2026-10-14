;; Host entry: (tick dt jump? duck?) → scene command list.

(define (tick dt jump? duck?)
    ;; Jump edge while dead restarts.
    (when (and dead? jump? (not prev-jump?))
        (reset))
    (step-player dt jump? duck?)
    (step-field dt)
    (draw-scene))
