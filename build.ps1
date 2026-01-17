# Build script for Speech-to-Text Rust application
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

Write-Host "Building Speech-to-Text application..." -ForegroundColor Cyan

# Clean previous builds
cargo clean

# Build in release mode
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild successful!" -ForegroundColor Green
    Write-Host "Executable: target\release\speech_to_text.exe" -ForegroundColor Yellow
    
    # Run the application
    Write-Host "`nStarting application..." -ForegroundColor Cyan
    .\target\release\speech_to_text.exe
} else {
    Write-Host "`nBuild failed!" -ForegroundColor Red
}
