using System;

class String__T__Examples
{
    static void Main__T__()
    {
        // __T__
        int user_Id = 42;
        string name = "Bob";

        // https://github.com/tree-sitter/tree-sitter-c-sharp/blob/1648e21b4f087963abf0101ee5221bb413107b07/src/node-types.json

        // interpolated_verbatim_string_text
        string interpolatedVerbatimString = $@"User {name} has the ID: {user_Id}";

        // interpolated_string_text
        string interpolatedStringText = $"Found user with ID: {user_Id}";

        // raw_string_literal
        string rawStringLiteral = """This is a
raw string
literal""";

        // string_literal
        string stringLiteral = "Alice";

        // verbatim_string_literal
        string verbatimStringLiteral = @"C:\Users\Alice\Documents";
    }
}
