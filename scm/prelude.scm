(define (_inc x) (+ (force x) 1))
(define (_dec x) (- (force x) 1))
(define _add (lambda (x0) (lambda (x1) (+ (force x0) (force x1)))))

(define _b
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force x0) ((force x1) x2))))))

(define _c
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force ((force x0) x2)) x1)))))

(define _f (lambda (x0) (lambda (x1) x1)))
(define _t (lambda (x0) (lambda (x1) x0)))

(define _cons
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((x2 x0) x1)))))

(define _car (lambda (x) ((force x) _t)))
(define _cdr (lambda (x) ((force x) _f)))
(define _nil (lambda (x) _t))

;; assertions
(define (assert desc actual expected)
  (if (eq? actual expected)
      #t
      (raise
       (string-append "test: " desc
                      ", expect: " (x->string expected)
                      ", but: " (x->string actual)))))

(assert "B Combinator" (((_b _inc) _dec) 42) 42)
(assert "C Combinator" (((_c _add) 1) 2) 3)
