(if_statement
  consequence: (block
    (return_statement (identifier)))
  alternative: (else_clause
    body: (block
      (return_statement (identifier))))) @cond
