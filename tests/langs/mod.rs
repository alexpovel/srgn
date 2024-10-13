use std::ops::Range;

use rstest::rstest;
use serde::{Deserialize, Serialize};
use srgn::scoping::langs::{c, csharp, go, hcl, python, typescript, LanguageScoper};
use srgn::scoping::scope::Scope;
use srgn::scoping::view::ScopedViewBuilder;
pub use tree_sitter::QueryError as TSQueryError;

/// A type that when serialized, will visually highlight the portions of a line which
/// were matched.
///
/// Single letter names to not disrupt alignment.
///
/// For example, when serialized as YAML, produces output such as
///
/// ```yaml
/// n: 156
/// l: "        print(f\"Loop iteration {i}\")\n"
/// m: "        ^^^^^                           "
/// ```
///
/// with `m` indicating the [`Scope::In`] part of the `l`ine.
#[derive(Debug, Serialize, Deserialize)]
struct InScopeLinePart {
    /// Line number.
    n: usize,
    /// Line itself, in original form.
    l: String,
    /// The string highlighting the matching. Has to be serialized *below*.
    m: String,
}

impl InScopeLinePart {
    fn new(line_number: usize, line_contents: String, match_: &str, span: Range<usize>) -> Self {
        // Split into the three components
        let (start, mid, end) = (
            &line_contents[..span.start],
            &line_contents[span.clone()],
            &line_contents[span.end..],
        );

        // ASSUMPTION is that `.escape_default()` will escape the same way (YAML, ...)
        // serialization does. Important for alignment to be correct.

        // Leading space
        let mut m = " ".repeat(start.escape_default().to_string().len());
        // The highlights for the line above.
        m.push_str(&"^".repeat(mid.escape_default().to_string().len()));
        // Trailing spaces; not strictly necessary but slightly nicer.
        m.push_str(&" ".repeat(end.escape_default().to_string().len()));

        assert_eq!(
            &line_contents[span], match_,
            "What is highlighted is what was matched"
        );

        Self {
            n: line_number,
            l: line_contents,
            m,
        }
    }
}

#[rstest]
#[case(
    "base.py_comments",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Comments.into()),
)]
#[case(
    "base.py_strings",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Strings.into()),
)]
#[case(
    "base.py_imports",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Imports.into()),
)]
#[case(
    "base.py_docstrings",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::DocStrings.into()),
)]
#[case(
    "base.py_function-names",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::FunctionNames.into()),
)]
#[case(
    "base.py_function-calls",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::FunctionCalls.into()),
)]
#[case(
    "base.py_class",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Class.into()),
)]
#[case(
    "base.py_def",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Def.into()),
)]
#[case(
    "base.py_async-def",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::AsyncDef.into()),
)]
#[case(
    "base.py_methods",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Methods.into()),
)]
#[case(
    "base.py_classmethods",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::ClassMethods.into()),
)]
#[case(
    "base.py_staticmethods",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::StaticMethods.into()),
)]
#[case(
    "base.py_with",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::With.into()),
)]
#[case(
    "base.py_try",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Try.into()),
)]
#[case(
    "base.py_lambda",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Lambda.into()),
)]
#[case(
    "base.py_globals",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Globals.into()),
)]
#[case(
    "base.py_variable_identifiers",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::VariableIdentifiers.into()),
)]
#[case(
    "base.py_types",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Types.into()),
)]
#[case(
    "base.py_identifiers",
    include_str!("python/base.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Identifiers.into()),
)]
#[case(
    "identifiers.py_identifiers",
    include_str!("python/identifiers.py"),
    python::CompiledQuery::new(&python::PreparedQuery::Identifiers.into()),
)]
#[case(
    "base.ts_strings",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Strings.into()),
)]
#[case(
    "base.ts_comments",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Comments.into()),
)]
#[case(
    "base.ts_imports",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Imports.into()),
)]
#[case(
    "base.ts_function",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Function.into()),
)]
#[case(
    "base.ts_async-function",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::AsyncFunction.into()),
)]
#[case(
    "base.ts_sync-function",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::SyncFunction.into()),
)]
#[case(
    "base.ts_method",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Method.into()),
)]
#[case(
    "base.ts_constructor",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Constructor.into()),
)]
#[case(
    "base.ts_class",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Class.into()),
)]
#[case(
    "base.ts_enum",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Enum.into()),
)]
#[case(
    "base.ts_interface",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Interface.into()),
)]
#[case(
    "base.ts_try-block",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::TryCatch.into()),
)]
#[case(
    "base.ts_var_decl",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::VarDecl.into()),
)]
#[case(
    "base.ts_let",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Let.into()),
)]
#[case(
    "base.ts_const",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Const.into()),
)]
#[case(
    "base.ts_var",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Var.into()),
)]
#[case(
    "base.ts_type-params",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::TypeParams.into()),
)]
#[case(
    "base.ts_type-alias",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::TypeAlias.into()),
)]
#[case(
    "base.ts_namespace",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Namespace.into()),
)]
#[case(
    "base.ts_export",
    include_str!("typescript/base.ts"),
    typescript::CompiledQuery::new(&typescript::PreparedQuery::Export.into()),
)]
#[case(
    "base.tf_variable-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Variable.into()),
)]
#[case(
    "base.tf_resource-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Resource.into()),
)]
#[case(
    "base.tf_data-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Data.into()),
)]
#[case(
    "base.tf_output-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Output.into()),
)]
#[case(
    "base.tf_provider-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Provider.into()),
)]
#[case(
    "base.tf_terraform-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Terraform.into()),
)]
#[case(
    "base.tf_locals-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Locals.into()),
)]
#[case(
    "base.tf_module-block",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Module.into()),
)]
#[case(
    "base.tf_variables",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Variables.into()),
)]
#[case(
    "base.tf_resource-types",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::ResourceTypes.into()),
)]
#[case(
    "base.tf_resource-names",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::ResourceNames.into()),
)]
#[case(
    "base.tf_data-names",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::DataNames.into()),
)]
#[case(
    "base.tf_data-sources",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::DataSources.into()),
)]
#[case(
    "base.tf_comments",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Comments.into()),
)]
#[case(
    "base.tf_strings",
    include_str!("hcl/base.tf"),
    hcl::CompiledQuery::new(&hcl::PreparedQuery::Strings.into()),
)]
#[case(
    "base.go_comments",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Comments.into()),
)]
#[case(
    "base.go_strings",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Strings.into()),
)]
#[case(
    "base.go_imports",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Imports.into()),
)]
#[case(
    "base.go_type-def",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::TypeDef.into()),
)]
#[case(
    "base.go_type-alias",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::TypeAlias.into()),
)]
#[case(
    "base.go_struct",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Struct.into()),
)]
#[case(
    "base.go_interface",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Interface.into()),
)]
#[case(
    "base.go_const",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Const.into()),
)]
#[case(
    "base.go_var",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Var.into()),
)]
#[case(
    "base.go_func",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Func.into()),
)]
#[case(
    "base.go_method",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Method.into()),
)]
#[case(
    "base.go_free-func",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::FreeFunc.into()),
)]
#[case(
    "base.go_init-func",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::InitFunc.into()),
)]
#[case(
    "base.go_type-params",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::TypeParams.into()),
)]
#[case(
    "base.go_defer",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Defer.into()),
)]
#[case(
    "base.go_select",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Select.into()),
)]
#[case(
    "base.go_go",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Go.into()),
)]
#[case(
    "base.go_switch",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Switch.into()),
)]
#[case(
    "base.go_labeled",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Labeled.into()),
)]
#[case(
    "base.go_goto",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::Goto.into()),
)]
#[case(
    "base.go_struct-tags",
    include_str!("go/base.go"),
    go::CompiledQuery::new(&go::PreparedQuery::StructTags.into()),
)]
#[case(
    "base.cs_strings",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Strings.into()),
)]
#[case(
    "base.cs_usings",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Usings.into()),
)]
#[case(
    "base.cs_comments",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Comments.into()),
)]
#[case(
    "base.cs_struct",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Struct.into()),
)]
#[case(
    "base.cs_enum",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Enum.into()),
)]
#[case(
    "base.cs_field",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Field.into()),
)]
#[case(
    "base.cs_attribute",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Attribute.into()),
)]
#[case(
    "base.cs_interface",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Interface.into()),
)]
#[case(
    "base.cs_class",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Class.into()),
)]
#[case(
    "base.cs_variable_declaration",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::VariableDeclaration.into()),
)]
#[case(
    "base.cs_property",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Property.into()),
)]
#[case(
    "base.cs_constructor",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Constructor.into()),
)]
#[case(
    "base.cs_destructor",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Destructor.into()),
)]
#[case(
    "base.cs_method",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Method.into()),
)]
#[case(
    "base.cs_identifier",
    include_str!("csharp/base.cs"),
    csharp::CompiledQuery::new(&csharp::PreparedQuery::Identifier.into()),
)]
#[case(
    "base.c_comments",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Comments.into()),
)]
#[case(
    "base.c_strings",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Strings.into()),
)]
#[case(
    "base.c_includes",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Includes.into()),
)]
#[case(
    "base.c_typedefs",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::TypeDef.into()),
)]
#[case(
    "base.c_enum",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Enum.into()),
)]
#[case(
    "base.c_struct",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Struct.into()),
)]
#[case(
    "base.c_variable",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Variable.into()),
)]
#[case(
    "base.c_function",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Function.into()),
)]
#[case(
    "base.c_function_definition",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::FunctionDef.into()),
)]
#[case(
    "base.c_function_declaration",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::FunctionDecl.into()),
)]
#[case(
    "base.c_switch",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Switch.into()),
)]
#[case(
    "base.c_if",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::If.into()),
)]
#[case(
    "base.c_for",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::For.into()),
)]
#[case(
    "base.c_while",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::While.into()),
)]
#[case(
    "base.c_do",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Do.into()),
)]
#[case(
    "base.c_union",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Union.into()),
)]
#[case(
    "base.c_identifier",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Identifier.into()),
)]
#[case(
    "base.c_declaration",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::Declaration.into()),
)]
#[case(
    "base.c_callexpr",
    include_str!("c/base.c"),
    c::CompiledQuery::new(&c::PreparedQuery::CallExpression.into()),
)]
fn test_language_scopers(
    #[case] snapshot_name: &str,
    #[case] contents: &str,
    #[case] lang: Result<impl LanguageScoper, TSQueryError>,
) {
    let lang = lang.expect("Building a Language for a test should not fail");
    let mut builder = ScopedViewBuilder::new(contents);
    builder.explode(&lang);
    let view = builder.build();

    // Collect only those lines which are in scope. This avoids enormous clutter from
    // out of scope items. The in-scope lines are *pinned* by lines numbers and
    // highlights/their span, so false-positives and confusion around multiple matches
    // per line are avoided.
    let inscope_parts: Vec<InScopeLinePart> = view
        .lines()
        .into_iter()
        .enumerate()
        .flat_map(|(i, line)| {
            let mut start = 0;

            line.scopes()
                .0
                .clone()
                .into_iter()
                .filter_map(move |scope| {
                    let contents: &str = (&scope).into();
                    let end = start + contents.len();
                    let span = start..end;
                    let part = InScopeLinePart::new(i + 1, line.to_string(), contents, span);
                    start = end;

                    match scope.0 {
                        Scope::In(..) => Some(part),
                        Scope::Out(..) => None,
                    }
                })
        })
        .collect();

    insta::assert_yaml_snapshot!(snapshot_name, inscope_parts);
}
