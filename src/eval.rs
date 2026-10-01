//! The evaluation driver: builds a snix evaluator, installs stubs, runs the
//! program and turns the result into output or diagnostics.

#[cfg(feature = "local-io")]
use snix_eval::StdIO;
use snix_eval::{EvalMode, EvalWarning, Evaluation, SourceCode, Value};

use crate::diagnostic::{Diagnostic, EvalError};
use crate::output;
use crate::program::{Wrap, compose, read_source};
use crate::stubs;
use crate::{EvalOptions, EvalOutput, Format, IoMode, Mode};

pub(crate) type Builder =
    snix_eval::EvaluationBuilder<'static, 'static, 'static, Box<dyn snix_eval::EvalIO>>;

/// Build a bare evaluator builder with all store/IO stubs installed.
///
/// Shared by `eval`, `check` and the REPL.
pub(crate) fn base_builder(io: IoMode) -> Result<Builder, EvalError> {
    install(Evaluation::builder_pure(), io)
}

/// Evaluate a Nix expression or file, returning the rendered output.
pub fn evaluate(options: &EvalOptions) -> Result<EvalOutput, EvalError> {
    let (base, location, _label) = read_source(&options.source)?;
    let mut program = compose(&base, &options.args, wrap_for(options.format))?;
    program.location = location;

    let source_map = SourceCode::default();
    let evaluation = base_builder(options.io)?
        .with_source_map(source_map.clone())
        .mode(to_eval_mode(options.mode))
        .nix_path(options.nix_path.clone())
        .build();

    let result = evaluation.evaluate(&program.code, program.location.clone());
    let warnings = collect_warnings(&result.warnings, &source_map, program.line_offset, &base);

    if !result.errors.is_empty() {
        let diagnostics = result
            .errors
            .iter()
            .map(|error| Diagnostic::from_snix_error(error, program.line_offset, &base))
            .collect();
        return Err(EvalError {
            diagnostics,
            warnings,
        });
    }

    let Some(value) = result.value else {
        return Err(EvalError {
            diagnostics: vec![Diagnostic::internal(
                "TSNIX-NO-VALUE",
                "evaluation produced no value",
            )],
            warnings,
        });
    };

    let text = render(&value, options)?;
    Ok(EvalOutput {
        format: options.format,
        text,
        warnings,
    })
}

/// Parse and compile without evaluating. Returns warnings on success.
pub fn check(options: &crate::CheckOptions) -> Result<Vec<Diagnostic>, EvalError> {
    let (base, location, _label) = read_source(&options.source)?;
    let mut program = compose(&base, &options.args, Wrap::Plain)?;
    program.location = location;

    let source_map = SourceCode::default();
    let evaluation = base_builder(options.io)?
        .with_source_map(source_map.clone())
        .mode(EvalMode::Strict)
        .build();

    let result = evaluation.compile_only(&program.code, program.location.clone());
    let warnings = collect_warnings(&result.warnings, &source_map, program.line_offset, &base);

    if result.errors.is_empty() {
        Ok(warnings)
    } else {
        let diagnostics = result
            .errors
            .iter()
            .map(|error| Diagnostic::from_snix_error(error, program.line_offset, &base))
            .collect();
        Err(EvalError {
            diagnostics,
            warnings,
        })
    }
}

fn install(mut builder: Builder, io: IoMode) -> Result<Builder, EvalError> {
    match io {
        IoMode::Local => {
            #[cfg(feature = "local-io")]
            {
                builder = builder.enable_impure(Some(Box::new(StdIO)));
            }
            #[cfg(not(feature = "local-io"))]
            {
                let _ = builder;
                return Err(EvalError::single(Diagnostic::internal(
                    "TSNIX-NO-LOCAL-IO",
                    "this build of tsnix was compiled without local I/O support",
                )));
            }
        }
        IoMode::None => {
            for &(name, source) in stubs::IO_STUBS {
                builder = builder.add_src_builtin(name, source);
            }
        }
    }
    for &(name, source) in stubs::STORE_STUBS {
        builder = builder.add_src_builtin(name, source);
    }
    #[cfg(feature = "stdlib")]
    {
        builder = builder.add_src_builtin("std", crate::stdlib::STDLIB_SRC);
    }
    Ok(builder)
}

pub(crate) fn to_eval_mode(mode: Mode) -> EvalMode {
    match mode {
        Mode::Strict => EvalMode::Strict,
        Mode::Lazy => EvalMode::Lazy,
    }
}

fn wrap_for(format: Format) -> Wrap {
    match format {
        Format::Json | Format::Nix => Wrap::ToJson,
        Format::Raw => Wrap::ToString,
    }
}

fn render(value: &Value, options: &EvalOptions) -> Result<String, EvalError> {
    match options.format {
        Format::Json => {
            let compact = string_of(value)?;
            if options.pretty {
                pretty_json(&compact)
            } else {
                Ok(compact)
            }
        }
        Format::Raw => string_of(value),
        Format::Nix => {
            // `--format nix` renders the JSON projection of the value. This
            // avoids unforceable thunks and mirrors `builtins.toJSON`'s
            // handling of paths and functions.
            let compact = string_of(value)?;
            let parsed: serde_json::Value = serde_json::from_str(&compact).map_err(|error| {
                EvalError::single(Diagnostic::internal("TSNIX-JSON", error.to_string()))
            })?;
            Ok(output::render_nix(&parsed))
        }
    }
}

fn string_of(value: &Value) -> Result<String, EvalError> {
    let string = value.to_str().map_err(|kind| {
        EvalError::single(Diagnostic::internal(
            "TSNIX-EXPECTED-STRING",
            format!("expected a string result: {kind}"),
        ))
    })?;
    string
        .as_str()
        .map(|text| text.to_owned())
        .map_err(|error| EvalError::single(Diagnostic::internal("TSNIX-UTF8", error.to_string())))
}

fn pretty_json(compact: &str) -> Result<String, EvalError> {
    let parsed: serde_json::Value = serde_json::from_str(compact).map_err(|error| {
        EvalError::single(Diagnostic::internal("TSNIX-JSON", error.to_string()))
    })?;
    serde_json::to_string_pretty(&parsed)
        .map_err(|error| EvalError::single(Diagnostic::internal("TSNIX-JSON", error.to_string())))
}

fn collect_warnings(
    warnings: &[EvalWarning],
    source_map: &SourceCode,
    line_offset: usize,
    source_text: &str,
) -> Vec<Diagnostic> {
    warnings
        .iter()
        .map(|warning| Diagnostic::from_warning(warning, source_map, line_offset, source_text))
        .collect()
}
