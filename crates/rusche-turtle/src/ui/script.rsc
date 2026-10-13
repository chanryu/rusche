; Script tab: syntax layouter, Run/Stop/Reset, editor, and Output.

(define (script-layouter text)
  (let ((spans (highlight text))
        (site (turtle-site text)))
    (if (eq? site false)
        spans
        (mark-site spans (car site) (cadr site) '(255 232 140)))))

(define (mark-site spans start end bg)
  ; Split on overlap with [start, end); leave non-overlapping spans unchanged.
  (define (loop spans out)
    (if (null? spans)
        out
        (let ((a (car (car spans)))
              (b (cadr (car spans)))
              (r (caddr (car spans)))
              (g (cadddr (car spans)))
              (bl (car (cddddr (car spans)))))
          (let ((mid0 (if (< a start) start a))
                (mid1 (if (< end b) end b)))
            (loop (cdr spans)
                  (append out
                    (if (< mid0 mid1)
                        (append
                          (if (< a mid0) (list (list a mid0 r g bl)) '())
                          (append
                            (list (list mid0 mid1 r g bl (car bg) (cadr bg) (caddr bg)))
                            (if (< mid1 b) (list (list mid1 b r g bl)) '())))
                        (list (list a b r g bl)))))))))
  (loop spans '()))

(define (draw-editor-controls)
  (horizontal (lambda ()
    (let ((running (turtle-running?)))
      (when (if running false (button "Run"))
        (turtle-run (buffer-text script)))
      (when (if running (button "Stop") false)
        (turtle-stop))
      (when (if running false (button "Reset"))
        (turtle-reset)
        (buffer-set! log-buf ""))
      (separator)
      (menu "Examples" (lambda ()
        (for-each
          (lambda (ex)
            (when (button (car ex))
              (buffer-set! script (cadr ex))
              (close-menu)))
          (turtle-examples))))))))

(define (draw-script-tab)
  (draw-editor-controls)
  (separator)
  (bottom-panel "log" 160 (lambda ()
    (label "Output" 'strong)
    (turtle-output log-buf)
    (scroll 'vertical 'stick-to-bottom (lambda ()
      (text-edit log-buf 'code 'read-only)))))
  (scroll 'both (lambda ()
    (text-edit script 'code 'rows 30 script-layouter))))
