(define-syntax _ap
  (syntax-rules ()
    ((_ fun arg) ((force fun) (lazy arg)))))

(define _bottom (lazy _bottom))

(define (_inc x) (+ (force x) 1))
(define (_dec x) (- (force x) 1))
(define _add (lambda (x0) (lambda (x1) (+ (force x0) (force x1)))))
(define _div (lambda (x0) (lambda (x1) (quotient (force x0) (force x1)))))
(define _mul (lambda (x0) (lambda (x1) (* (force x0) (force x1)))))
(define _neg (lambda (x0) (- (force x0))))

(define _eq
  (lambda (x0)
    (lambda (x1)
      (if (= (force x0) (force x1))
          _t
          _f))))
(define _lt
  (lambda (x0)
    (lambda (x1)
      (if (< (force x0) (force x1))
          _t
          _f))))

(define _b
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force x0) (lazy ((force x1) x2)))))))

(define _c
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force ((force x0) x2)) x1)))))

(define _s
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force ((force x0) x2)) (lazy ((force x1) x2)))))))

(define _i (lambda (x0) x0))

(define _f (lambda (x0) (lambda (x1) x1)))
(define _t (lambda (x0) (lambda (x1) x0)))

(define _cons
  (lambda (x0)
    (lambda (x1)
      (lambda (x2)
        ((force ((force x2) x0)) x1)))))

(define _car (lambda (x) ((force x) _t)))
(define _cdr (lambda (x) ((force x) _f)))
(define _nil (lambda (x) _t))

;; MEMO: nil が自作されると死ぬ
(define _isnil (lambda (x) (if (eq? (force x) _nil) _t _f)))

;; ap ap s ap ap c ap eq 0 1 ap ap b ap mul 2 ap ap b pwr2 ap add -1
(define _pwr2
  (lazy ((force ((force _s) ((force ((force _c) ((force _eq) 0))) 1)))
         ((force ((force _b) ((force _mul) 2)))
          ((force ((force _b) _pwr2)) ((force _add) -1))))))

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
(assert "S Combinator (1)" (((_s _add) _inc) 1) 3)
(assert "S Combinator (2)" (((_s _mul) (_add 1)) 6) 42)
(assert "Power of Two (0)" (force ((force _pwr2) 0)) 1)
(assert "Power of Two (1)" (force ((force _pwr2) 1)) 2)
(assert "Power of Two (2)" (force ((force _pwr2) 2)) 4)
(assert "Power of Two (3)" (force ((force _pwr2) 3)) 8)
(assert "Power of Two (8)" (force ((force _pwr2) 8)) 256)

(assert "AP (1)" (_ap _inc 0) 1)
(assert "AP (2)" (_ap (_ap _add 1) 1) 2)

(assert "Lazy Eval Test" (force (_ap _car (_ap (_ap _cons 1) _bottom))) 1)
