(defmacro (backwards *args)
    `(begin ,@(reverse args)))

(backwards
    (println "uno")
    (println "dos")
    (println "tres"))
