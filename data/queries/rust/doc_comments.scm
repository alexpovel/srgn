(
    (line_comment)+ @line
    (#match? @line "^//(/|!)")
)
