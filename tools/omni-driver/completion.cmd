@echo off
:: Omni driver shell completion for CMD
:: Usage:
%1   1. Save this file as omni-driver-completion.bat
::   2. Add to CMD auto-run:
::      reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /d "C:\path\to\omni-driver-completion.bat" /f

@echo off
setlocal

:: Define omni-driver commands and options
set OMNI_COMMANDS=--help --native --native --run -o --output -O --opt-level -h

:loop
if "%1"=="" goto :end
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help
if "%1"=="--native" goto :native
if "%1"=="-o" goto :output
if "%1"=="--output" goto :output
if "%1"=="-O" goto :opt-level
if "%1"=="--opt-level" goto :opt-level
if "%1"=="--run" goto :run

:shift
shift
goto :loop

:help
echo Omni Compiler Driver - Available options:
echo   --help, -h          Show this help message
echo   --native            Emit native object and link to executable
echo   --run               Run the linked executable after linking (implies --native)
echo   -o, --output <FILE> Set output artifact path
echo   -O, --opt-level <NUM> Set optimization level (0-3)
goto :end

:native
echo Native compilation mode selected
goto :end

:output
shift
if "%1"=="" goto :end
echo Output file: %1
goto :end

:opt-level
shift
if "%1"=="" goto :end
echo Optimization level: %1
goto :end

:run
echo Run after linking enabled
goto :end

:end
endlocal
