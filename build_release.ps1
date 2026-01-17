$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
Set-Location "C:\Users\Ashmil P\Desktop\Handoff\speech_to_text"

# Build optimized release
$env:CARGO_TARGET_DIR = "C:\temp\voxmagic_final_build"
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build Successful!"
    Copy-Item -Path "C:\temp\voxmagic_final_build\release\VoxMagic.exe" -Destination "C:\Users\Ashmil P\Desktop\Handoff\VoxMagic.exe" -Force
}
