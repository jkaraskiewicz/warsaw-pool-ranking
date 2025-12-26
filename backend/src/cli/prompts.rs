use std::io::{self, Write};
use anyhow::Result;

/// Prompt user for yes/no confirmation
pub fn confirm(message: &str, default_yes: bool) -> Result<bool> {
    let prompt_suffix = if default_yes { " [Y/n]: " } else { " [y/N]: " };

    print!("{}{}", message, prompt_suffix);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() {
        return Ok(default_yes);
    }

    Ok(matches!(trimmed.as_str(), "y" | "yes"))
}

/// Confirm destructive operation with enhanced warning
pub fn confirm_destructive(resource_name: &str, operation: &str) -> Result<bool> {
    println!("\n⚠️  WARNING: Destructive Operation ⚠️");
    println!("You are about to {} all {}.", operation, resource_name);
    println!("This action CANNOT be undone.");
    println!();

    confirm(
        &format!("Type 'yes' to confirm {} {}", operation, resource_name),
        false,
    )
}
