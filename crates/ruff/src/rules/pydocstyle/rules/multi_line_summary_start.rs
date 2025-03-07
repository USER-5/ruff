use ruff_text_size::{TextRange, TextSize};
use rustpython_parser::ast::Ranged;

use ruff_diagnostics::{AlwaysAutofixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::str::{is_triple_quote, leading_quote};
use ruff_python_semantic::{Definition, Member};
use ruff_python_whitespace::{NewlineWithTrailingNewline, UniversalNewlineIterator};

use crate::checkers::ast::Checker;
use crate::docstrings::Docstring;
use crate::registry::{AsRule, Rule};

/// ## What it does
/// Checks for docstring summary lines that are not positioned on the first
/// physical line of the docstring.
///
/// ## Why is this bad?
/// [PEP 257] recommends that multi-line docstrings consist of "a summary line
/// just like a one-line docstring, followed by a blank line, followed by a
/// more elaborate description."
///
/// The summary line should be located on the first physical line of the
/// docstring, immediately after the opening quotes.
///
/// This rule may not apply to all projects; its applicability is a matter of
/// convention. By default, this rule is enabled when using the `google`
/// convention, and disabled when using the `numpy` and `pep257` conventions.
///
/// For an alternative, see [D213].
///
/// ## Example
/// ```python
/// def sort_list(l: list[int]) -> list[int]:
///     """
///     Return a sorted copy of the list.
///
///     Sort the list in ascending order and return a copy of the result using the
///     bubble sort algorithm.
///     """
/// ```
///
/// Use instead:
/// ```python
/// def sort_list(l: list[int]) -> list[int]:
///     """Return a sorted copy of the list.
///
///     Sort the list in ascending order and return a copy of the result using the bubble
///     sort algorithm.
///     """
/// ```
///
/// [D213]: https://beta.ruff.rs/docs/rules/multi-line-summary-second-line
#[violation]
pub struct MultiLineSummaryFirstLine;

impl AlwaysAutofixableViolation for MultiLineSummaryFirstLine {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Multi-line docstring summary should start at the first line")
    }

    fn autofix_title(&self) -> String {
        "Remove whitespace after opening quotes".to_string()
    }
}

/// ## What it does
/// Checks for docstring summary lines that are not positioned on the second
/// physical line of the docstring.
///
/// ## Why is this bad?
/// [PEP 257] recommends that multi-line docstrings consist of "a summary line
/// just like a one-line docstring, followed by a blank line, followed by a
/// more elaborate description."
///
/// The summary line should be located on the second physical line of the
/// docstring, immediately after the opening quotes and the blank line.
///
/// This rule may not apply to all projects; its applicability is a matter of
/// convention. By default, this rule is disabled when using the `google`,
/// `numpy`, and `pep257` conventions.
///
/// For an alternative, see [D212].
///
/// ## Example
/// ```python
/// def sort_list(l: list[int]) -> list[int]:
///     """Return a sorted copy of the list.
///
///     Sort the list in ascending order and return a copy of the result using the
///     bubble sort algorithm.
///     """
/// ```
///
/// Use instead:
/// ```python
/// def sort_list(l: list[int]) -> list[int]:
///     """
///     Return a sorted copy of the list.
///
///     Sort the list in ascending order and return a copy of the result using the bubble
///     sort algorithm.
///     """
/// ```
///
/// [D212]: https://beta.ruff.rs/docs/rules/multi-line-summary-first-line
#[violation]
pub struct MultiLineSummarySecondLine;

impl AlwaysAutofixableViolation for MultiLineSummarySecondLine {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Multi-line docstring summary should start at the second line")
    }

    fn autofix_title(&self) -> String {
        "Insert line break and indentation after opening quotes".to_string()
    }
}

/// D212, D213
pub(crate) fn multi_line_summary_start(checker: &mut Checker, docstring: &Docstring) {
    let contents = docstring.contents;
    let body = docstring.body();

    if NewlineWithTrailingNewline::from(body.as_str())
        .nth(1)
        .is_none()
    {
        return;
    };
    let mut content_lines = UniversalNewlineIterator::with_offset(contents, docstring.start());

    let Some(first_line) = content_lines
        .next()
         else
    {
        return;
    };

    if is_triple_quote(&first_line) {
        if checker.enabled(Rule::MultiLineSummaryFirstLine) {
            let mut diagnostic = Diagnostic::new(MultiLineSummaryFirstLine, docstring.range());
            if checker.patch(diagnostic.kind.rule()) {
                // Delete until first non-whitespace char.
                for line in content_lines {
                    if let Some(end_column) = line.find(|c: char| !c.is_whitespace()) {
                        diagnostic.set_fix(Fix::automatic(Edit::deletion(
                            first_line.end(),
                            line.start() + TextSize::try_from(end_column).unwrap(),
                        )));
                        break;
                    }
                }
            }
            checker.diagnostics.push(diagnostic);
        }
    } else {
        if checker.enabled(Rule::MultiLineSummarySecondLine) {
            let mut diagnostic = Diagnostic::new(MultiLineSummarySecondLine, docstring.range());
            if checker.patch(diagnostic.kind.rule()) {
                let mut indentation = String::from(docstring.indentation);
                let mut fixable = true;
                if !indentation.chars().all(char::is_whitespace) {
                    fixable = false;

                    // If the docstring isn't on its own line, look at the statement indentation,
                    // and add the default indentation to get the "right" level.
                    if let Definition::Member(Member { stmt, .. }) = &docstring.definition {
                        let stmt_line_start = checker.locator.line_start(stmt.start());
                        let stmt_indentation = checker
                            .locator
                            .slice(TextRange::new(stmt_line_start, stmt.start()));

                        if stmt_indentation.chars().all(char::is_whitespace) {
                            indentation.clear();
                            indentation.push_str(stmt_indentation);
                            indentation.push_str(checker.stylist.indentation());
                            fixable = true;
                        }
                    };
                }

                if fixable {
                    let prefix = leading_quote(contents).unwrap();
                    // Use replacement instead of insert to trim possible whitespace between leading
                    // quote and text.
                    let repl = format!(
                        "{}{}{}",
                        checker.stylist.line_ending().as_str(),
                        indentation,
                        first_line.strip_prefix(prefix).unwrap().trim_start()
                    );

                    diagnostic.set_fix(Fix::automatic(Edit::replacement(
                        repl,
                        body.start(),
                        first_line.end(),
                    )));
                }
            }
            checker.diagnostics.push(diagnostic);
        }
    }
}
