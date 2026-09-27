# Downloads the Whisper base English model used for local transcription.
$ErrorActionPreference = "Stop"

$ModelDir = Join-Path $PSScriptRoot "..\models"
$ModelPath = Join-Path $ModelDir "ggml-base.en.bin"
$Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"

if (Test-Path $ModelPath) {
    Write-Host "Model already exists at $ModelPath"
    exit 0
}

New-Item -ItemType Directory -Path $ModelDir -Force | Out-Null
Write-Host "Downloading ggml-base.en.bin (~140 MB) from $Url ..."
Invoke-WebRequest -Uri $Url -OutFile $ModelPath
Write-Host "Saved to $ModelPath"
