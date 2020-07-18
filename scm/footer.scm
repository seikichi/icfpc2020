;; (use util.combinations)

;; (define (iterate ps)
;;   (if (not (null? ps))
;;       (begin
;;         (let* ((x (caar ps))
;;                (y (cadar ps))
;;                (result (_usual (_ap (_ap z0
;;                                          (_ap (_ap _cons 0) 0))
;;                                     (_ap (_ap _cons x) y)))))
;;           (if (> (car result) 0)
;;               (print (string-append "flag = " (x->string (car result))
;;                                     ", (when x = " (x->string x)
;;                                     ", y = " (x->string y) ")"
;;                                     )))
;;           (iterate (cdr ps))))))

;; (define ps (cartesian-product (list (iota 21 -10) (iota 21 -10))))

;; (iterate ps)

;; (define result (_ap (_ap z0 _nil) (_ap (_ap _cons 0) 0)))
;; (0 (0) 0 ())
;; (1 (4) 0 ())
;; (galaxy nil (?, ?)) -> flag は 0 ばっかり...
;; 返り値の newState を見ると...
;; (0 (0) 0 ())
;; 適当に (1 (11) 1 ()) とか変えると...
;;;; (1 (11 -1) 1 ()) -> big

(define (_unusual x)
  (cond ((number? x) x)
        ((eq? x #t) _t)
        ((eq? x #f) _f)
        ((null? x) _nil)
        (else (_ap (_ap _cons (_unusual (car x))) (_unusual (cdr x))))))

(define data (_unusual '(2 (1 -1) 1 ())))
(define point (_unusual '(0 . 0)))

;; (define data _nil)
;; (define point (_ap (_ap _cons 0) 0))
;; (define point (_ap (_ap _cons 42) 42))
(define result (_ap (_ap z0 data) point))

(define (pretty-print-sexp s)
  (define (do-indent level)
    (dotimes (_ level) (write-char #\space)))
  (define (pp-parenl)
    (write-char #\())
  (define (pp-parenr)
    (write-char #\)))
  (define (pp-atom e prefix)
    (when prefix (write-char #\space))
    (write e))
  (define (pp-list s level prefix)
    (and prefix (do-indent level))
    (pp-parenl)
    (let loop ((s s)
               (prefix #f))
      (if (null? s)
          (pp-parenr)
          (let1 e (car s)
            (if (list? e)
                (begin (and prefix (newline))
                       (pp-list e (+ level 1) prefix))
                (pp-atom e prefix))
            (loop (cdr s) #t)))))
  (if (list? s)
      (pp-list s 0 #f)
      (write s))
  (newline))

(print (force (_ap _car result)))
;; (print (force (_ap _cadr result)))
;; (print (force (_ap _caddr result)))
(pretty-print-sexp (_usual result))
;; (pretty-print-sexp (_usual data))

;; (print ((force z0) _nil))
;; (print (force ((force z2000) 8)))

(_multipledraw (caddr (_usual result)))
