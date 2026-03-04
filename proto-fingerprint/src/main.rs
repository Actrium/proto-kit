use anyhow::Result;
use clap::Parser;
use proto_fingerprint::spec::{Compatibility, Spec};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "proto-sign")]
#[command(about = "Check Protobuf file compatibility and generate semantic fingerprints")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Compare two .proto files for compatibility")]
    Compare {
        #[arg(help = "Path to the old .proto file")]
        old_file: PathBuf,
        #[arg(help = "Path to the new .proto file")]
        new_file: PathBuf,
        #[arg(long, help = "Show detailed breaking change analysis")]
        detailed: bool,
    },
    #[command(about = "Generate semantic fingerprint for a .proto file")]
    Fingerprint {
        #[arg(help = "Path to the .proto file")]
        file: PathBuf,
    },
    #[command(about = "Check for breaking changes using Buf-compatible rules")]
    Breaking {
        #[arg(help = "Path to the old .proto file")]
        old_file: PathBuf,
        #[arg(help = "Path to the new .proto file")]
        new_file: PathBuf,
        #[arg(long, help = "Output format", value_enum, default_value = "text")]
        format: OutputFormat,
        #[arg(long, help = "Rules to use (comma-separated)")]
        use_rules: Option<String>,
        #[arg(long, help = "Categories to use (comma-separated)")]
        use_categories: Option<String>,
        #[arg(long, help = "Rules to exclude (comma-separated)")]
        except_rules: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Compare {
            old_file,
            new_file,
            detailed,
        } => {
            let old_content = fs::read_to_string(&old_file).map_err(|e| {
                anyhow::anyhow!("Failed to read old file '{}': {}", old_file.display(), e)
            })?;
            let new_content = fs::read_to_string(&new_file).map_err(|e| {
                anyhow::anyhow!("Failed to read new file '{}': {}", new_file.display(), e)
            })?;

            let old_spec = Spec::try_from(old_content.as_str())?;
            let new_spec = Spec::try_from(new_content.as_str())?;

            let compatibility = old_spec.compare_with(&new_spec);

            match compatibility {
                Compatibility::Green => {
                    println!("Green: Files are semantically identical");
                    if detailed {
                        let breaking_result = old_spec.check_breaking_changes(&new_spec);
                        println!(
                            "Detailed analysis: {} rules executed, {} breaking changes found",
                            breaking_result.executed_rules.len(),
                            breaking_result.changes.len()
                        );
                    }
                    std::process::exit(0);
                }
                Compatibility::Yellow => {
                    println!("Yellow: New file is backward-compatible with old file");
                    if detailed {
                        let breaking_result = old_spec.check_breaking_changes(&new_spec);
                        println!(
                            "Detailed analysis: {} rules executed, {} breaking changes found",
                            breaking_result.executed_rules.len(),
                            breaking_result.changes.len()
                        );
                        if breaking_result.has_breaking_changes {
                            println!(
                                "Note: Some breaking changes were detected but are considered acceptable for backward compatibility"
                            );
                        }
                    }
                    std::process::exit(0);
                }
                Compatibility::Red => {
                    println!("Red: Breaking change detected");
                    if detailed {
                        let breaking_result = old_spec.check_breaking_changes(&new_spec);
                        println!(
                            "Detailed analysis: {} rules executed, {} breaking changes found",
                            breaking_result.executed_rules.len(),
                            breaking_result.changes.len()
                        );
                        for change in &breaking_result.changes {
                            println!("  - {}: {}", change.rule_id, change.message);
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Fingerprint { file } => {
            let content = fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file.display(), e))?;

            let fingerprint = proto_fingerprint::generate_fingerprint(&content)?;
            println!("{fingerprint}");
        }
        Commands::Breaking {
            old_file,
            new_file,
            format,
            use_rules,
            use_categories,
            except_rules,
        } => {
            use proto_fingerprint::compat::BreakingConfig;

            let old_content = fs::read_to_string(&old_file).map_err(|e| {
                anyhow::anyhow!("Failed to read old file '{}': {}", old_file.display(), e)
            })?;
            let new_content = fs::read_to_string(&new_file).map_err(|e| {
                anyhow::anyhow!("Failed to read new file '{}': {}", new_file.display(), e)
            })?;

            let old_spec = Spec::try_from(old_content.as_str())?;
            let new_spec = Spec::try_from(new_content.as_str())?;

            // Build configuration
            let mut config = BreakingConfig::default();

            if let Some(rules) = use_rules {
                config.use_rules = rules.split(',').map(|s| s.trim().to_string()).collect();
                config.use_categories.clear(); // Clear default categories when specific rules are used
            }

            if let Some(categories) = use_categories {
                config.use_categories = categories
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }

            if let Some(except) = except_rules {
                config.except_rules = except.split(',').map(|s| s.trim().to_string()).collect();
            }

            let breaking_result = old_spec.check_breaking_changes_with_config(&new_spec, &config);

            match format {
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&breaking_result)?;
                    println!("{json}");
                }
                OutputFormat::Text => {
                    if breaking_result.has_breaking_changes {
                        println!("Breaking changes detected:");
                        for change in &breaking_result.changes {
                            println!("  [{}] {}", change.rule_id, change.message);
                            println!(
                                "    Location: {} ({})",
                                change.location.element_name, change.location.element_type
                            );
                            if let Some(prev_loc) = &change.previous_location {
                                println!(
                                    "    Previous: {} ({})",
                                    prev_loc.element_name, prev_loc.element_type
                                );
                            }
                            println!("    Categories: {}", change.categories.join(", "));
                            println!();
                        }
                        println!("Summary:");
                        println!(
                            "  Total breaking changes: {}",
                            breaking_result.changes.len()
                        );
                        println!("  Rules executed: {}", breaking_result.executed_rules.len());
                        if !breaking_result.failed_rules.is_empty() {
                            println!(
                                "  Rules failed: {} ({})",
                                breaking_result.failed_rules.len(),
                                breaking_result.failed_rules.join(", ")
                            );
                        }
                    } else {
                        println!("No breaking changes detected.");
                        println!("Rules executed: {}", breaking_result.executed_rules.len());
                    }
                }
            }

            if breaking_result.has_breaking_changes {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
