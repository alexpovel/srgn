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
        string interpolatedVerbatimString = $@"User {name} has __T__the ID: {user_Id}";

        // interpolated_string_text
        string interpolatedStringText = $"Found user __T__with ID: {user_Id}";

        // raw_string_literal
        string rawStringLiteral = """This __T__is a
raw string__T__
literal""";

        // string_literal
        string stringLiteral = "Ali__T__ce";

        // verbatim_string_literal
        string verbatimStringLiteral = @"C:\Users\Alice__T__\Documents";
    }
}
