use covariant_syntax::SourceFile;

use crate::dag::Dag;
use crate::error::IrError;

/// Lower a parsed source file into an IR DAG.
pub fn lower(_source: &SourceFile) -> (Dag, Vec<IrError>) {
    (Dag::new(), Vec::new())
}
