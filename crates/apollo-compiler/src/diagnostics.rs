// NOTE @lrlna: only syntax errors currently have the source data.
//
// TODO: figure out a nice way of going back to the AST and get its source data
// given a current Value, which will make sure the rest of the diagnostics have
// source data.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    Error(ErrorDiagnostic),
    Warning(WarningDiagnostic),
    Hint(HintDiagnostic),
    Suggestion(SuggestionDiagnostic),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ErrorDiagnostic {
    MissingIdent(String),
    SingleRootField(String),
    UniqueOperationDefinition {
        message: String,
        operation: String,
    },
    UndefinedVariable {
        message: String,
        variable: String,
    },
    SyntaxError {
        message: String,
        data: String,
        index: usize,
    },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WarningDiagnostic {
    UnusedVariable { message: String, variable: String },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum HintDiagnostic {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SuggestionDiagnostic {}