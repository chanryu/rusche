;; Field: scroll, spawn, collide, score.
;; Chromium offline.js: clusters, type-dup limit, bird minSpeed / speedOffset, CLEAR_TIME.

(define (unit-size kind)
    (cond ((eq? kind 'cactus-s) (list CACTUS-S-W CACTUS-S-H))
          ((eq? kind 'cactus-l) (list CACTUS-L-W CACTUS-L-H))
          ((eq? kind 'bird) (list BIRD-W BIRD-H))
          (else (list 0 0))))

(define (cadddr lst) (car (cdr (cddr lst))))
(define (cdddr lst) (cdr (cddr lst)))
(define (cddddr lst) (cdr (cdddr lst)))

;; Obstacle: (kind x y size speed-offset flap-t)
(define (obs-kind obs) (car obs))
(define (obs-x obs) (cadr obs))
(define (obs-y obs) (caddr obs))
(define (obs-size obs) (cadddr obs))
(define (obs-voff obs) (car (cddddr obs)))
(define (obs-flap obs) (cadr (cddddr obs)))

(define (obs-width obs)
    (* (car (unit-size (obs-kind obs))) (obs-size obs)))

(define (obs-height obs)
    (cadr (unit-size (obs-kind obs))))

(define (cactus-y kind)
    (cond ((eq? kind 'cactus-s) (- GROUND-Y CACTUS-S-H))
          ((eq? kind 'cactus-l) (- GROUND-Y CACTUS-L-H))
          (else 0)))

;; PTERODACTYL yPos [100, 75, 50]
(define (bird-y-pick)
    (let ((r (rand)))
        (cond ((< r 0.33) 50)
              ((< r 0.66) 75)
              (else 100))))

(define (multi-speed kind)
    (cond ((eq? kind 'cactus-s) SMALL-MULTI-SPEED)
          ((eq? kind 'cactus-l) LARGE-MULTI-SPEED)
          (else 99999)))

;; Roll 1..MAX-CLUSTER; clamp to 1 until multipleSpeed (birds always 1).
(define (pick-size kind)
    (if (eq? kind 'bird)
        1
        (let ((n (+ 1 (rand-int 0 MAX-CLUSTER))))
            (if (< speed (multi-speed kind)) 1 n))))

(define (bird-voff)
    (if (< (rand) 0.5) BIRD-SPEED-OFFSET (- 0 BIRD-SPEED-OFFSET)))

;; True if last MAX-TYPE-DUP kinds were already `kind`.
(define (would-duplicate? kind)
    (if (< (length recent-kinds) MAX-TYPE-DUP)
        false
        (let ((a (car recent-kinds))
              (b (cadr recent-kinds)))
            (and (eq? a kind) (eq? b kind)))))

(define (note-kind kind)
    (set! recent-kinds (cons kind recent-kinds))
    (when (> (length recent-kinds) MAX-TYPE-DUP)
        (set! recent-kinds (list (car recent-kinds) (cadr recent-kinds)))))

;; Retries if type-dup would fire.
(define (pick-kind)
    (define (roll)
        (let ((r (rand)))
            (cond ((and (>= speed BIRD-MIN-SPEED) (< r 0.33)) 'bird)
                  ((< r 0.66) 'cactus-s)
                  (else 'cactus-l))))
    (define (try n)
        (let ((k (roll)))
            (if (and (would-duplicate? k) (> n 0))
                (try (- n 1))
                k)))
    (try 8))

(define (make-obstacle)
    (let ((kind (pick-kind)))
        (note-kind kind)
        (list kind
              WIDTH
              (if (eq? kind 'bird) (bird-y-pick) (cactus-y kind))
              (pick-size kind)
              (if (eq? kind 'bird) (bird-voff) 0)
              0)))

;; Gap ≈ width*speed/60 + MIN-GAP*GAP-COEFF, up to 1.5×.
(define (gap-for obs)
    (let ((min-gap (+ (* (obs-width obs) (/ speed 60))
                      (* MIN-GAP GAP-COEFF)))
          (max-gap (* (+ (* (obs-width obs) (/ speed 60))
                         (* MIN-GAP GAP-COEFF))
                      1.5)))
        (rand-range min-gap (max (+ min-gap 40) max-gap))))

(define (aabb-hit ax ay aw ah bx by bw bh)
    (and (< ax (+ bx bw))
         (< bx (+ ax aw))
         (< ay (+ by bh))
         (< by (+ ay ah))))

;; Inset a bit — full sprite AABB feels unfair.
(define (obstacle-hitbox obs)
    (list (+ (obs-x obs) 2)
          (+ (obs-y obs) 2)
          (- (obs-width obs) 4)
          (- (obs-height obs) 4)))

(define (hits-dino? obs)
    (let ((d (dino-hitbox))
          (o (obstacle-hitbox obs)))
        (aabb-hit (car d) (cadr d) (caddr d) (cadddr d)
                  (car o) (cadr o) (caddr o) (cadddr o))))

;; Per-bird flap timer (Chromium Obstacle.timer).
(define (step-flap obs dt)
    (if (eq? (obs-kind obs) 'bird)
        (+ (obs-flap obs) dt)
        0))

(define (move-obstacles dt)
    (set! obstacles
        (filter (lambda (obs) (> (+ (obs-x obs) (obs-width obs)) -10))
                (map (lambda (obs)
                        (list (obs-kind obs)
                              (- (obs-x obs) (* (+ speed (obs-voff obs)) dt))
                              (obs-y obs)
                              (obs-size obs)
                              (obs-voff obs)
                              (step-flap obs dt)))
                     obstacles))))

(define (maybe-spawn dt)
    (set! spawn-cd (- spawn-cd dt))
    (when (and started? (not dead?) (<= spawn-cd 0))
        (let ((obs (make-obstacle)))
            (set! obstacles (cons obs obstacles))
            (set! spawn-cd (/ (gap-for obs) speed)))))

(define (move-clouds dt)
    (let ((dx (* speed dt 0.35)))
        (set! clouds
            (map (lambda (c)
                    (let ((x (- (car c) dx))
                          (y (cadr c)))
                        (if (< x -60)
                            (list (+ WIDTH (rand-range 0 80)) (+ 20 (* (rand) 50)))
                            (list x y))))
                 clouds))))

(define (check-collisions)
    (when (and started? (not dead?))
        (when (not (null? (filter hits-dino? obstacles)))
            (set! dead? true)
            (when (> score hi-score)
                (set! hi-score score)))))

(define (update-score dt)
    (when (and started? (not dead?))
        (set! score (+ score (* speed dt SCORE-PER-PX)))
        (set! speed (min MAX-SPEED (+ speed (* SPEED-ACCEL dt))))
        (let ((mark (* BLINK-EVERY (floor (/ score BLINK-EVERY)))))
            (when (and (> mark 0) (> mark last-blink-mark))
                (set! last-blink-mark mark)
                (set! blink-t BLINK-SECS)))
        (when (> blink-t 0)
            (set! blink-t (- blink-t dt))))
    (set! ground-scroll (% (+ ground-scroll (* speed dt)) 24)))

(define (night?)
    (= (% (floor (/ score NIGHT-BAND)) 2) 1))

(define (step-field dt)
    (when (and started? (not dead?))
        (move-obstacles dt)
        (maybe-spawn dt)
        (move-clouds dt)
        (check-collisions)
        (update-score dt))
    (when (and started? dead?)
        (when (> blink-t 0)
            (set! blink-t (- blink-t dt)))))
