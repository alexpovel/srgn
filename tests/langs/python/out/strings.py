name = "Alice"
ag__T__e = 30
f_string = f"name: {name}, age: {ag__T__e}"

multiline_f_string = f"""This is a
multiline{f_string} string
spanning several lines"""

raw_string = r"This is a raw string with no special treatment for \n"
bytes_string = b"This is a bytes string"
bytes_string = rf"This is a raw f-string with {raw_string}"
