use clap::{Parser, Subcommand};
use pheno_cli_base::CliRunnable;
use anyhow::Result;

/// PlayCua CLI wrapper — delegates to pheno-cli-base patterns.
#[derive(Parser, Debug)]
#[command(name = "playcua", about = "PlayCua hand-rolled CLI wrapper")]
pub struct PlayCuaCli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Run the PlayCua application.
    Run,
    /// Run tests.
    Test,
    /// Build the project.
    Build,
}

impl CliRunnable for PlayCuaCli {
    fn run(self) -> Result<()> {
        match self.cmd {
            Commands::Run => {
                println!("Placeholder: running PlayCua");
                Ok(())
            }
            Commands::Test => {
                println!("Placeholder: running tests");
                Ok(())
            }
            Commands::Build => {
                println!("Placeholder: building project");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parses_subcommands() {
        let run = PlayCuaCli::parse_from(["playcua", "run"]);
        assert!(matches!(run.cmd, Commands::Run));

        let test = PlayCuaCli::parse_from(["playcua", "test"]);
        assert!(matches!(test.cmd, Commands::Test));

        let build = PlayCuaCli::parse_from(["playcua", "build"]);
        assert!(matches!(build.cmd, Commands::Build));
    }

    #[test]
    fn run_subcommand_prints_placeholder() {
        let cli = PlayCuaCli {
            cmd: Commands::Run,
        };
        // CliRunnable::run should succeed for Run.
        cli.run().expect("run subcommand should not fail");
    }
}
