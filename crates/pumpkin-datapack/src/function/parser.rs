use crate::DatapackError;

/// A single command line within a function.
#[derive(Debug, Clone)]
pub enum FunctionLine {
    /// A command to execute (without leading `/`).
    Command(String),
    /// A macro template line: `$(variable_name)command_text`.
    MacroTemplate(String),
    /// A comment (starts with `#`).
    Comment,
    /// Empty line.
    Empty,
}

/// A parsed `.mcfunction` file.
#[derive(Debug, Clone)]
pub enum MCFunction {
    /// Simple pre-parsed list of commands (no macro variables).
    PlainText { commands: Vec<String> },
    /// Function with macro substitutions (`$(name)` placeholders).
    Macro { template: String },
}

/// Parse the raw content of a `.mcfunction` file.
///
/// Rules (vanilla parity):
/// - Lines starting with `#` are comments
/// - Lines starting with `$` are macro template lines
/// - Lines starting with `/` are **errors** (no leading slash in mcfunction)
/// - Empty lines are skipped
/// - Trailing `\` continues to the next line
/// - Max 2,000,000 characters per command
pub fn parse_function(raw: &str) -> Result<MCFunction, DatapackError> {
    let mut commands = Vec::new();
    let mut has_macro = false;
    let mut current_line = String::new();
    let mut line_count = 0;

    for line in raw.lines() {
        line_count += 1;

        // Handle line continuation (trailing backslash)
        if line.ends_with('\\') && line.len() > 1 {
            current_line.push_str(&line[..line.len() - 1]);
            current_line.push('\n');
            continue;
        }
        if !current_line.is_empty() {
            current_line.push_str(line);
            let full = std::mem::take(&mut current_line);
            process_line(&full, &mut commands, &mut has_macro, line_count)?;
            continue;
        }

        process_line(line, &mut commands, &mut has_macro, line_count)?;
    }

    // Handle trailing continuation
    if !current_line.is_empty() {
        process_line(&current_line, &mut commands, &mut has_macro, line_count)?;
    }

    if has_macro {
        Ok(MCFunction::Macro {
            template: raw.to_string(),
        })
    } else {
        Ok(MCFunction::PlainText { commands })
    }
}

fn process_line(
    line: &str,
    commands: &mut Vec<String>,
    has_macro: &mut bool,
    line_num: i32,
) -> Result<(), DatapackError> {
    let trimmed = line.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }

    if trimmed.starts_with('$') {
        *has_macro = true;
        commands.push(trimmed.to_string());
        return Ok(());
    }

    if trimmed.starts_with('/') {
        return Err(DatapackError::Function(format!(
            "Line {line_num}: commands in .mcfunction must not start with '/'"
        )));
    }

    if trimmed.len() > 2_000_000 {
        return Err(DatapackError::Function(format!(
            "Line {line_num}: command exceeds 2,000,000 character limit"
        )));
    }

    commands.push(trimmed.to_string());
    Ok(())
}
