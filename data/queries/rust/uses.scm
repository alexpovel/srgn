
; Match any (wildcard `_`) `argument`, which includes:
;
; - `scoped_identifier`
; - `scoped_use_list`
; - `use_wildcard`
; - `use_as_clause`
;
; all at once.
[
    (use_declaration
        argument: (_) @use
    )
]

