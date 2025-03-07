use ruff_text_size::TextRange;
use rustpython_parser::ast::{self, CmpOp, Expr, ExprContext, Ranged, Stmt, UnaryOp};

use ruff_diagnostics::{AlwaysAutofixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_semantic::ScopeKind;

use crate::checkers::ast::Checker;
use crate::registry::AsRule;

/// ## What it does
/// Checks for negated `==` operators.
///
/// ## Why is this bad?
/// Negated `==` operators are less readable than `!=` operators. When testing
/// for non-equality, it is more common to use `!=` than `==`.
///
/// ## Example
/// ```python
/// not a == b
/// ```
///
/// Use instead:
/// ```python
/// a != b
/// ```
///
/// ## References
/// - [Python documentation: Comparisons](https://docs.python.org/3/reference/expressions.html#comparisons)
#[violation]
pub struct NegateEqualOp {
    left: String,
    right: String,
}

impl AlwaysAutofixableViolation for NegateEqualOp {
    #[derive_message_formats]
    fn message(&self) -> String {
        let NegateEqualOp { left, right } = self;
        format!("Use `{left} != {right}` instead of `not {left} == {right}`")
    }

    fn autofix_title(&self) -> String {
        "Replace with `!=` operator".to_string()
    }
}

/// ## What it does
/// Checks for negated `!=` operators.
///
/// ## Why is this bad?
/// Negated `!=` operators are less readable than `==` operators, as they avoid a
/// double negation.
///
/// ## Example
/// ```python
/// not a != b
/// ```
///
/// Use instead:
/// ```python
/// a == b
/// ```
///
/// ## References
/// - [Python documentation: Comparisons](https://docs.python.org/3/reference/expressions.html#comparisons)
#[violation]
pub struct NegateNotEqualOp {
    left: String,
    right: String,
}

impl AlwaysAutofixableViolation for NegateNotEqualOp {
    #[derive_message_formats]
    fn message(&self) -> String {
        let NegateNotEqualOp { left, right } = self;
        format!("Use `{left} == {right}` instead of `not {left} != {right}`")
    }

    fn autofix_title(&self) -> String {
        "Replace with `==` operator".to_string()
    }
}

/// ## What it does
/// Checks for double negations (i.e., multiple `not` operators).
///
/// ## Why is this bad?
/// A double negation is redundant and less readable than omitting the `not`
/// operators entirely.
///
/// ## Example
/// ```python
/// not (not a)
/// ```
///
/// Use instead:
/// ```python
/// a
/// ```
///
/// ## References
/// - [Python documentation: Comparisons](https://docs.python.org/3/reference/expressions.html#comparisons)
#[violation]
pub struct DoubleNegation {
    expr: String,
}

impl AlwaysAutofixableViolation for DoubleNegation {
    #[derive_message_formats]
    fn message(&self) -> String {
        let DoubleNegation { expr } = self;
        format!("Use `{expr}` instead of `not (not {expr})`")
    }

    fn autofix_title(&self) -> String {
        let DoubleNegation { expr } = self;
        format!("Replace with `{expr}`")
    }
}

fn is_dunder_method(name: &str) -> bool {
    matches!(
        name,
        "__eq__" | "__ne__" | "__lt__" | "__le__" | "__gt__" | "__ge__"
    )
}

fn is_exception_check(stmt: &Stmt) -> bool {
    let Stmt::If(ast::StmtIf {test: _, body, orelse: _, range: _ })= stmt else {
        return false;
    };
    if body.len() != 1 {
        return false;
    }
    if matches!(body[0], Stmt::Raise(_)) {
        return true;
    }
    false
}

/// SIM201
pub(crate) fn negation_with_equal_op(
    checker: &mut Checker,
    expr: &Expr,
    op: UnaryOp,
    operand: &Expr,
) {
    if !matches!(op, UnaryOp::Not) {
        return;
    }
    let Expr::Compare(ast::ExprCompare { left, ops, comparators, range: _}) = operand else {
        return;
    };
    if !matches!(&ops[..], [CmpOp::Eq]) {
        return;
    }
    if is_exception_check(checker.semantic().stmt()) {
        return;
    }

    // Avoid flagging issues in dunder implementations.
    if let ScopeKind::Function(ast::StmtFunctionDef { name, .. })
    | ScopeKind::AsyncFunction(ast::StmtAsyncFunctionDef { name, .. }) =
        &checker.semantic().scope().kind
    {
        if is_dunder_method(name) {
            return;
        }
    }

    let mut diagnostic = Diagnostic::new(
        NegateEqualOp {
            left: checker.generator().expr(left),
            right: checker.generator().expr(&comparators[0]),
        },
        expr.range(),
    );
    if checker.patch(diagnostic.kind.rule()) {
        let node = ast::ExprCompare {
            left: left.clone(),
            ops: vec![CmpOp::NotEq],
            comparators: comparators.clone(),
            range: TextRange::default(),
        };
        diagnostic.set_fix(Fix::automatic(Edit::range_replacement(
            checker.generator().expr(&node.into()),
            expr.range(),
        )));
    }
    checker.diagnostics.push(diagnostic);
}

/// SIM202
pub(crate) fn negation_with_not_equal_op(
    checker: &mut Checker,
    expr: &Expr,
    op: UnaryOp,
    operand: &Expr,
) {
    if !matches!(op, UnaryOp::Not) {
        return;
    }
    let Expr::Compare(ast::ExprCompare { left, ops, comparators, range: _}) = operand else {
        return;
    };
    if !matches!(&ops[..], [CmpOp::NotEq]) {
        return;
    }
    if is_exception_check(checker.semantic().stmt()) {
        return;
    }

    // Avoid flagging issues in dunder implementations.
    if let ScopeKind::Function(ast::StmtFunctionDef { name, .. })
    | ScopeKind::AsyncFunction(ast::StmtAsyncFunctionDef { name, .. }) =
        &checker.semantic().scope().kind
    {
        if is_dunder_method(name) {
            return;
        }
    }

    let mut diagnostic = Diagnostic::new(
        NegateNotEqualOp {
            left: checker.generator().expr(left),
            right: checker.generator().expr(&comparators[0]),
        },
        expr.range(),
    );
    if checker.patch(diagnostic.kind.rule()) {
        let node = ast::ExprCompare {
            left: left.clone(),
            ops: vec![CmpOp::Eq],
            comparators: comparators.clone(),
            range: TextRange::default(),
        };
        diagnostic.set_fix(Fix::suggested(Edit::range_replacement(
            checker.generator().expr(&node.into()),
            expr.range(),
        )));
    }
    checker.diagnostics.push(diagnostic);
}

/// SIM208
pub(crate) fn double_negation(checker: &mut Checker, expr: &Expr, op: UnaryOp, operand: &Expr) {
    if !matches!(op, UnaryOp::Not) {
        return;
    }
    let Expr::UnaryOp(ast::ExprUnaryOp { op: operand_op, operand, range: _ }) = operand else {
        return;
    };
    if !matches!(operand_op, UnaryOp::Not) {
        return;
    }

    let mut diagnostic = Diagnostic::new(
        DoubleNegation {
            expr: checker.generator().expr(operand),
        },
        expr.range(),
    );
    if checker.patch(diagnostic.kind.rule()) {
        if checker.semantic().in_boolean_test() {
            diagnostic.set_fix(Fix::suggested(Edit::range_replacement(
                checker.generator().expr(operand),
                expr.range(),
            )));
        } else if checker.semantic().is_builtin("bool") {
            let node = ast::ExprName {
                id: "bool".into(),
                ctx: ExprContext::Load,
                range: TextRange::default(),
            };
            let node1 = ast::ExprCall {
                func: Box::new(node.into()),
                args: vec![*operand.clone()],
                keywords: vec![],
                range: TextRange::default(),
            };
            diagnostic.set_fix(Fix::suggested(Edit::range_replacement(
                checker.generator().expr(&node1.into()),
                expr.range(),
            )));
        };
    }
    checker.diagnostics.push(diagnostic);
}
