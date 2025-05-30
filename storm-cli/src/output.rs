use crate::{OutputFormat, GLOBAL_OPTS};
use colored::*;
use serde::Serialize;
use std::io;

/// Print output according to the global format settings
pub fn print_output<T: Serialize>(data: &T) -> Result<(), io::Error> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if opts.quiet {
        return Ok(());
    }

    match opts.output {
        OutputFormat::Json => print_json(data),
        OutputFormat::Csv => print_csv(data),
        OutputFormat::Text => Ok(()), // Text output is handled by individual commands
    }
}

/// Print JSON output
pub fn print_json<T: Serialize>(data: &T) -> Result<(), io::Error> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// Print CSV output (simplified - you might want to use the csv crate)
pub fn print_csv<T: Serialize>(data: &T) -> Result<(), io::Error> {
    // This is a simplified version - for real CSV output, use the csv crate
    let json_value = serde_json::to_value(data)?;

    if let serde_json::Value::Array(arr) = json_value {
        // Print headers (assuming all objects have same fields)
        if let Some(serde_json::Value::Object(obj)) = arr.first() {
            println!("{}", obj.keys().cloned().collect::<Vec<_>>().join(","));
        }

        // Print rows
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                let values: Vec<String> = obj
                    .values()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .collect();
                println!("{}", values.join(","));
            }
        }
    }

    Ok(())
}

/// Print verbose message (only if verbose mode is on)
pub fn verbose_println(level: u8, message: &str) {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if !opts.quiet && opts.verbose >= level {
        eprintln!("{} {}", "[VERBOSE]".dimmed(), message);
    }
}

/// Check if we should use color
pub fn use_color() -> bool {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");
    !opts.no_color && opts.output == OutputFormat::Text
}
