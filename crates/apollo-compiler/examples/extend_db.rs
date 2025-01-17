use std::fmt;

use apollo_compiler::{
    database::{AstStorage, DocumentStorage, HirStorage, InputStorage},
    AstDatabase, DocumentDatabase, HirDatabase, InputDatabase,
};
use miette::{Diagnostic, Report, SourceSpan};
use thiserror::Error;

/// A small example public API for this linter example.
pub struct Linter {
    pub db: LinterDatabase,
}

impl Linter {
    /// Create a new instance of Linter.
    pub fn new(input: &str) -> Self {
        let mut db = LinterDatabase::default();
        let input = input.to_string();
        db.set_input(input);
        Self { db }
    }

    /// Runs lints.
    pub fn lint(&self) -> Vec<LintDiagnostic> {
        self.db.lint()
    }
}

// Includes all the necessary database's storage units that will now be
// accessible from LinterDatabase.
#[salsa::database(
    DocumentStorage,
    InputStorage,
    AstStorage,
    HirStorage,
    LintValidationStorage
)]
#[derive(Default)]
pub struct LinterDatabase {
    pub storage: salsa::Storage<LinterDatabase>,
}

impl salsa::Database for LinterDatabase {}

// This is important if your LinterDatabase storage needs to be accessed from in
// a multi-threaded environment. You can drop this otherwise.
impl salsa::ParallelDatabase for LinterDatabase {
    fn snapshot(&self) -> salsa::Snapshot<LinterDatabase> {
        salsa::Snapshot::new(LinterDatabase {
            storage: self.storage.snapshot(),
        })
    }
}

// We need this upcast to upcast LinterDatabase query groups to Apollo
// Compiler's DocumentDatabase.
pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

impl Upcast<dyn DocumentDatabase> for LinterDatabase {
    fn upcast(&self) -> &(dyn DocumentDatabase + 'static) {
        self
    }
}

// LintValidation database. It's based on four other Apollo Compiler databases.
// We are also making sure we can upcast to DocumentDatabase with Upcast<dyn
// DocumentDatabase>.
#[salsa::query_group(LintValidationStorage)]
pub trait LintValidation:
    Upcast<dyn DocumentDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    // Define any queries that should be part of this database.
    fn lint(&self) -> Vec<LintDiagnostic>;
    fn capitalised_definitions(&self) -> Vec<LintDiagnostic>;
}

// Implemenatation of the queries defined above. The lint query calls on
// capitalised_definitions query. You ideally want queries to be based on other
// queries.
fn lint(db: &dyn LintValidation) -> Vec<LintDiagnostic> {
    let mut lints = Vec::new();
    lints.extend(db.capitalised_definitions());

    lints
}

fn capitalised_definitions(db: &dyn LintValidation) -> Vec<LintDiagnostic> {
    let lints: Vec<LintDiagnostic> = db
        .db_definitions()
        .iter()
        .filter_map(|def| {
            if !def.name()?.chars().next()?.is_uppercase() {
                if let Some(node) = def.name_src()?.ast_node(db.upcast()) {
                    let offset: usize = node.text_range().start().into();
                    let len: usize = node.text_range().len().into();

                    Some(LintDiagnostic::CapitalisedDefinitions(
                        CapitalisedDefinitions {
                            src: db.input(),
                            definition: (offset, len).into(),
                        },
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    lints
}

// Lint Diagnostics.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum LintDiagnostic {
    CapitalisedDefinitions(CapitalisedDefinitions),
}

// This is specific to ensure lints are pretty printed. We are using `miette`'s
// Report feature here.
impl LintDiagnostic {
    pub fn report(&self) -> Report {
        match self {
            LintDiagnostic::CapitalisedDefinitions(lint) => Report::new(lint.clone()),
        }
    }
}

// The pretty printed reports are only available with `Display`, otherwise lints
// will be just structs, which is nice if you wish your tools to be further
// processed.
impl fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.report())
    }
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("definitions should be capitalised")]
#[diagnostic(code("graphql linter diagnostic"))]
pub struct CapitalisedDefinitions {
    #[source_code]
    pub src: String,

    #[label = "capitalise this definition"]
    pub definition: SourceSpan,
}

fn main() {
    let input = r#"
type query {
  topProducts: Product
  customer: User
}

type product {
  type: String
  price(setPrice: Int): Int
}

type user {
  id: ID
  name: String
  profilePic(size: Int): URL
}

scalar url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
    "#;

    let linter = Linter::new(input);
    let lints = linter.lint();

    // Display lints.
    for lint in &lints {
        println!("{}", lint)
    }
}
