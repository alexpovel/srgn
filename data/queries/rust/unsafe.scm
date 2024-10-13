
[
    (
        (trait_item) @ti (#match? @ti "^unsafe")
    )
    (
        (impl_item) @ii (#match? @ii "^unsafe")
    )
    (function_item
        (function_modifiers) @funcmods
        (#match? @funcmods "unsafe")
    ) @function_item
    (function_signature_item
        (function_modifiers) @funcmods
        (#match? @funcmods "unsafe")
    ) @function_signature_item
    (unsafe_block) @block
] @unsafe
