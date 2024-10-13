[
    (line_comment)+ @line
    (block_comment)
    (#not-match? @line "^///")
]
@comment
