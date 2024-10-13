
; Any attribute which matches aka contains `test`, preceded or
; followed by more attributes, eventually preceded by a function.
; The anchors of `.` ensure nothing but the items we're after occur
; in between.
(
    (attribute_item)*
    .
    (attribute_item (attribute) @_SRGN_IGNORE.attr (#match? @_SRGN_IGNORE.attr "test"))
    .
    (attribute_item)*
    .
    (function_item) @func
)
