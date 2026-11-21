# Omni driver shell completion for PowerShell
# Usage:
#   1. Save this file as omni-driver-completion.ps1
#   2. Add the following to your PowerShell profile ($PROFILE):
#      . "C:\path\to\omni-driver-completion.ps1"
#   3. Or source it directly: . ./omni-driver-completion.ps1

using namespace System.Management.Automation

Register-ArgumentCompleter -CommandName omni-driver -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $completions = @()

    # Main subcommands and options
    $mainOptions = @(
        "--help", "-h",
        "--native", "-o", "--output", "-O", "--opt-level", "--run"
    )

    # If no word yet or starting with -, show main options
    if ($wordToComplete -eq '' -or $wordToComplete.StartsWith('-')) {
        foreach ($opt in $mainOptions) {
            if ($opt.StartsWith($wordToComplete)) {
                $completions += [System.Management.Automation.CompletionResult]::new($opt, $opt, [System.Management.Automation.CommandCompletionType]::ParameterValue, $opt)
            }
        }
    }

    $completions
}
