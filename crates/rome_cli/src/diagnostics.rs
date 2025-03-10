use rome_console::fmt::Formatter;
use rome_console::markup;
use rome_diagnostics::adapters::{IoError, PicoArgsError};
use rome_diagnostics::{
    Advices, Category, Diagnostic, DiagnosticTags, Error, Location, LogCategory, Severity, Visit,
};
use rome_service::WorkspaceError;
use std::process::{ExitCode, Termination};
use std::{env::current_exe, fmt::Debug};

fn command_name() -> String {
    current_exe()
        .ok()
        .and_then(|path| Some(path.file_name()?.to_str()?.to_string()))
        .unwrap_or_else(|| String::from("rome"))
}

/// A diagnostic that is emitted when running rome via CLI.
///
/// When displaying the diagnostic,
#[derive(Debug)]
pub enum CliDiagnostic {
    /// Returned when it is called with a subcommand it doesn't know
    UnknownCommand(UnknownCommand),
    /// Return by the help command when it is called with a subcommand it doesn't know
    UnknownCommandHelp(UnknownCommandHelp),
    /// Returned when the value of a command line argument could not be parsed
    ParseError(ParseDiagnostic),
    /// Returned when the CLI  doesn't recognize a command line argument
    UnexpectedArgument(UnexpectedArgument),
    /// Returned when a required argument is not present in the command line
    MissingArgument(MissingArgument),
    /// Returned when a subcommand is called without any arguments
    EmptyArguments(EmptyArguments),
    /// Returned when a subcommand is called with an unsupported combination of arguments
    IncompatibleArguments(IncompatibleArguments),
    /// Returned by a traversal command when error diagnostics were emitted
    CheckError(CheckError),
    /// When an argument is higher than the expected maximum
    OverflowNumberArgument(OverflowNumberArgument),
    /// Wrapper for an underlying `rome_service` error
    WorkspaceError(WorkspaceError),
    /// Wrapper for an underlying `std::io` error
    IoError(IoDiagnostic),
    /// The daemon is not running
    ServerNotRunning(ServerNotRunning),
    /// The end configuration (`rome.json` + other options) is incompatible with the command
    IncompatibleEndConfiguration(IncompatibleEndConfiguration),
    /// No files processed during the file system traversal
    NoFilesWereProcessed(NoFilesWereProcessed),
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "Unknown command {command_name}",
        message("Unknown command "<Emphasis>{self.command_name}</Emphasis>)
    ),
)]
pub struct UnknownCommand {
    command_name: String,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
category = "flags/invalid",
    severity = Error,
    message(
        description = "Cannot print help for unknown command {command_name}",
        message("Cannot print help for unknown command "<Emphasis>{self.command_name}</Emphasis>)
    ),
)]
pub struct UnknownCommandHelp {
    command_name: String,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "Cannot parse the argument {argument}",
        message("Cannot parse the argument "<Emphasis>{self.argument}</Emphasis>)
    ),
)]
pub struct ParseDiagnostic {
    argument: String,
    #[source]
    source: Option<Error>,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "Unrecognized option {argument}",
        message("Unrecognized option "<Emphasis>{self.argument}</Emphasis>".")
    ),
)]
pub struct UnexpectedArgument {
    argument: String,
    #[advice]
    help: CliAdvice,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "Unrecognized option {argument}",
        message("Missing argument "<Emphasis>{self.argument}</Emphasis>)
    ),
)]
pub struct MissingArgument {
    argument: String,
    #[advice]
    advice: CliAdvice,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message = "Empty arguments"
)]
pub struct EmptyArguments;

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "Incompatible arguments {first_argument} and {second_argument}",
        message("Incompatible arguments "<Emphasis>{self.first_argument}</Emphasis>" and "<Emphasis>{self.second_argument}</Emphasis>)
    )
)]
pub struct IncompatibleArguments {
    first_argument: String,
    second_argument: String,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "internalError/io",
    severity = Error,
    message = "Some errors were emitted while running checks"
)]
pub struct CheckError;

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "flags/invalid",
    severity = Error,
    message(
        description = "The value of the argument {argument} is too high, maximum accepted {maximum}",
        message("The value of the argument "<Emphasis>{self.argument}</Emphasis>" is too high, maximum accepted "{{self.maximum}})
    )
)]
pub struct OverflowNumberArgument {
    argument: String,
    maximum: u16,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "internalError/io",
    severity = Error,
    message = "Errors occurred while executing I/O operations."
)]
pub struct IoDiagnostic {
    #[source]
    source: Option<Error>,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "internalError/io",
    severity = Error,
    message = "No running instance of the Rome daemon server was found."
)]
// TODO: add advice
pub struct ServerNotRunning;

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "internalError/io",
    severity = Error,
    message(
        description = "The combination of configuration and arguments is invalid: \n{reason}",
        message("The combination of configuration and arguments is invalid: \n"{{&self.reason}})
    )
)]
pub struct IncompatibleEndConfiguration {
    reason: String,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "internalError/io",
    severity = Error,
    message = "No files were processed in the specified paths."
)]
pub struct NoFilesWereProcessed;

/// Advices for the [CliDiagnostic]
#[derive(Debug, Default)]
struct CliAdvice {
    /// Used to print the help command
    sub_command: String,
}

impl CliAdvice {
    fn new_with_help(sub_command: impl Into<String>) -> Self {
        Self {
            sub_command: sub_command.into(),
        }
    }
}

impl Advices for CliAdvice {
    fn record(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        let command_name = command_name();
        let help_sub_command = format!("{} {} --help", command_name, &self.sub_command);
        visitor.record_log(
            LogCategory::Info,
            &markup! { "Type the following command for more information" },
        )?;
        visitor.record_command(&help_sub_command)?;

        Ok(())
    }
}

impl CliDiagnostic {
    /// Returned when a subcommand is called with an unsupported combination of arguments
    pub fn incompatible_arguments(
        first_argument: impl Into<String>,
        second_argument: impl Into<String>,
    ) -> Self {
        Self::IncompatibleArguments(IncompatibleArguments {
            first_argument: first_argument.into(),
            second_argument: second_argument.into(),
        })
    }

    /// To throw when there's been an error while parsing an argument
    pub fn parse_error(argument: impl Into<String>, source: pico_args::Error) -> Self {
        Self::ParseError(ParseDiagnostic {
            source: Some(Error::from(PicoArgsError::from(source))),
            argument: argument.into(),
        })
    }

    /// Returned when it is called with a subcommand it doesn't know
    pub fn unknown_command(command: impl Into<String>) -> Self {
        Self::UnknownCommand(UnknownCommand {
            command_name: command.into(),
        })
    }

    /// Returned when a subcommand is called without any arguments
    pub fn empty_arguments() -> Self {
        Self::EmptyArguments(EmptyArguments)
    }

    /// Returned when a required argument is not present in the command line
    pub fn missing_argument(argument: impl Into<String>, subcommand: impl Into<String>) -> Self {
        Self::MissingArgument(MissingArgument {
            argument: argument.into(),
            advice: CliAdvice::new_with_help(subcommand),
        })
    }

    /// When no files were processed while traversing the file system
    pub fn no_files_processed() -> Self {
        Self::NoFilesWereProcessed(NoFilesWereProcessed)
    }

    /// Returned when the CLI  doesn't recognize a command line argument
    pub fn unexpected_argument(argument: impl Into<String>, subcommand: impl Into<String>) -> Self {
        Self::UnexpectedArgument(UnexpectedArgument {
            argument: argument.into(),
            help: CliAdvice::new_with_help(subcommand),
        })
    }

    /// When there's been error inside the workspace
    pub fn workspace_error(error: WorkspaceError) -> Self {
        Self::WorkspaceError(error)
    }

    /// An I/O error
    pub fn io_error(error: std::io::Error) -> Self {
        Self::IoError(IoDiagnostic {
            source: Some(Error::from(IoError::from(error))),
        })
    }

    /// Emitted when errors were emitted while traversing the file system
    pub fn check_error() -> Self {
        Self::CheckError(CheckError)
    }

    /// Emitted when the server is not running
    pub fn server_not_running() -> Self {
        Self::ServerNotRunning(ServerNotRunning)
    }

    /// Emitted when the end configuration (`rome.json` file + CLI arguments + LSP configuration)
    /// results in a combination of options that doesn't allow to run the command correctly.
    ///
    /// A reason needs to be provided
    pub fn incompatible_end_configuration(reason: impl Into<String>) -> Self {
        Self::IncompatibleEndConfiguration(IncompatibleEndConfiguration {
            reason: reason.into(),
        })
    }

    /// Emitted when an argument value is greater than the allowed value
    pub fn overflown_argument(argument: impl Into<String>, maximum: u16) -> Self {
        Self::OverflowNumberArgument(OverflowNumberArgument {
            argument: argument.into(),
            maximum,
        })
    }

    /// Return by the help command when it is called with a subcommand it doesn't know
    pub fn new_unknown_help(command: impl Into<String>) -> Self {
        Self::UnknownCommandHelp(UnknownCommandHelp {
            command_name: command.into(),
        })
    }
}

impl Diagnostic for CliDiagnostic {
    fn category(&self) -> Option<&'static Category> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.category(),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.category(),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.category(),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.category(),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.category(),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.category(),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.category(),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.category(),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.category(),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.category(),
            CliDiagnostic::IoError(diagnostic) => diagnostic.category(),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.category(),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.category(),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.category(),
        }
    }

    fn tags(&self) -> DiagnosticTags {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.tags(),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.tags(),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.tags(),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.tags(),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.tags(),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.tags(),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.tags(),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.tags(),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.tags(),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.tags(),
            CliDiagnostic::IoError(diagnostic) => diagnostic.tags(),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.tags(),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.tags(),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.tags(),
        }
    }

    fn severity(&self) -> Severity {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.severity(),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.severity(),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.severity(),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.severity(),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.severity(),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.severity(),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.severity(),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.severity(),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.severity(),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.severity(),
            CliDiagnostic::IoError(diagnostic) => diagnostic.severity(),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.severity(),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.severity(),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.severity(),
        }
    }

    fn location(&self) -> Location<'_> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.location(),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.location(),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.location(),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.location(),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.location(),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.location(),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.location(),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.location(),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.location(),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.location(),
            CliDiagnostic::IoError(diagnostic) => diagnostic.location(),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.location(),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.location(),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.location(),
        }
    }

    fn message(&self, fmt: &mut Formatter<'_>) -> std::io::Result<()> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::IoError(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.message(fmt),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.message(fmt),
        }
    }

    fn description(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::IoError(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.description(fmt),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.description(fmt),
        }
    }

    fn advices(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::IoError(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.advices(visitor),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.advices(visitor),
        }
    }

    fn verbose_advices(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => {
                diagnostic.verbose_advices(visitor)
            }
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::IoError(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.verbose_advices(visitor),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => {
                diagnostic.verbose_advices(visitor)
            }
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.verbose_advices(visitor),
        }
    }

    fn source(&self) -> Option<&dyn Diagnostic> {
        match self {
            CliDiagnostic::UnknownCommand(diagnostic) => diagnostic.source(),
            CliDiagnostic::UnknownCommandHelp(diagnostic) => diagnostic.source(),
            CliDiagnostic::ParseError(diagnostic) => diagnostic.source(),
            CliDiagnostic::UnexpectedArgument(diagnostic) => diagnostic.source(),
            CliDiagnostic::MissingArgument(diagnostic) => diagnostic.source(),
            CliDiagnostic::EmptyArguments(diagnostic) => diagnostic.source(),
            CliDiagnostic::IncompatibleArguments(diagnostic) => diagnostic.source(),
            CliDiagnostic::CheckError(diagnostic) => diagnostic.source(),
            CliDiagnostic::OverflowNumberArgument(diagnostic) => diagnostic.source(),
            CliDiagnostic::WorkspaceError(diagnostic) => diagnostic.source(),
            CliDiagnostic::IoError(diagnostic) => diagnostic.source(),
            CliDiagnostic::ServerNotRunning(diagnostic) => diagnostic.source(),
            CliDiagnostic::IncompatibleEndConfiguration(diagnostic) => diagnostic.source(),
            CliDiagnostic::NoFilesWereProcessed(diagnostic) => diagnostic.source(),
        }
    }
}

impl From<WorkspaceError> for CliDiagnostic {
    fn from(error: WorkspaceError) -> Self {
        CliDiagnostic::workspace_error(error)
    }
}

impl From<std::io::Error> for CliDiagnostic {
    fn from(error: std::io::Error) -> Self {
        CliDiagnostic::io_error(error)
    }
}

impl Termination for CliDiagnostic {
    fn report(self) -> ExitCode {
        let severity = self.severity();
        if severity >= Severity::Error {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CliDiagnostic;

    #[test]
    fn termination_diagnostic_size() {
        assert_eq!(
            std::mem::size_of::<CliDiagnostic>(),
            88,
            "you successfully decreased the size of the diagnostic!"
        )
    }
}
