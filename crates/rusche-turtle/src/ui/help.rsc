; Help panel: Turtle API reference with click-to-insert forms.

(define help-sections
  (list
    (list "Motion"
      (list "(forward dist)  ; fd" "Move forward (draws if pen down)" "(forward 50)")
      (list "(back dist)  ; bk" "Move backward" "(back 50)")
      (list "(left deg)  ; lt" "Turn left" "(left 90)")
      (list "(right deg)  ; rt" "Turn right" "(right 90)")
      (list "(goto x y)" "Absolute move" "(goto 0 0)")
      (list "(setheading deg)  ; seth" "Absolute heading" "(setheading 90)")
      (list "(home)" "Origin, face north" "(home)"))
    (list "Pen & canvas"
      (list "(penup)  ; pu" "Lift pen" "(penup)")
      (list "(pendown)  ; pd" "Lower pen" "(pendown)")
      (list "(pencolor \"blue\")" "Name or r g b" "(pencolor \"blue\")")
      (list "(pensize w)" "Stroke width" "(pensize 2)")
      (list "(clear)" "Reset canvas" "(clear)")
      (list "(hideturtle) / (showturtle)" "Sprite visibility" "(hideturtle)"))
    (list "Queries & I/O"
      (list "(position)" "→ (x y)" "(position)")
      (list "(heading)" "→ degrees (0 east, 90 north)" "(heading)")
      (list "(pen?)" "→ pen down?" "(pen?)")
      (list "(write …) / (writeln …)" "Output log" "(writeln \"hi\")")
      (list "(random)" "→ [0, 1)" "(random)")
      (list "(repeat n body…)" "Loop body n times" "(repeat 4 (forward 50) (right 90))"))))

(define (draw-help)
  (label "Turtle API" 'strong)
  (label "Origin at center, y up. Heading 0° = east, 90° = north." 'small 'weak)
  (space 4)
  (label "Click a form to append it to the script." 'small 'weak)
  (separator)
  (scroll 'vertical (lambda ()
    (for-each
      (lambda (section)
        (space 6)
        (label (car section) 'strong)
        (for-each
          (lambda (op)
            (let ((form (car op))
                  (blurb (cadr op))
                  (insert (caddr op)))
              (when (button form)
                (append-script insert))
              (hover (str-append "Insert `" insert "`"))
              (label blurb 'small 'weak)
              (space 2)))
          (cdr section)))
      help-sections))))
