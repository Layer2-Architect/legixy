<#
.SYNOPSIS
  legixy installer for Windows (PowerShell 5+).

.DESCRIPTION
  1. Downloads a prebuilt legixy release archive (x86_64-windows.zip) from GitHub Releases.
  2. Installs it under a prefix (default: %LOCALAPPDATA%\legixy) and adds <prefix>\bin to the user PATH.
  3. Downloads the ONNX embedding model (paraphrase-multilingual-MiniLM-L12-v2) from the
     Hugging Face Hub and points LGX_MODELS_DIR (user env) at it, so the semantic layer works.

  The model is NOT shipped inside the release archive (~500 MB, separate license); this
  script is how you obtain it. Use -NoModel to skip it.

.PARAMETER Repo     GitHub "owner/repo" to download from. Or set $env:LEGIXY_REPO.
.PARAMETER Version  Release tag, or "latest" (default).
.PARAMETER Prefix   Install prefix (default: %LOCALAPPDATA%\legixy).
.PARAMETER NoModel  Skip the embedding-model download.

.EXAMPLE
  irm <raw-url>/install.ps1 | iex
  powershell -ExecutionPolicy Bypass -File install.ps1 -Repo Layer2-Architect/legixy -Version v0.4.0
#>
param(
  [string]$Repo    = $(if ($env:LEGIXY_REPO)    { $env:LEGIXY_REPO }    else { "Layer2-Architect/legixy" }),
  [string]$Version = $(if ($env:LEGIXY_VERSION) { $env:LEGIXY_VERSION } else { "latest" }),
  [string]$Prefix  = $(if ($env:LEGIXY_PREFIX)  { $env:LEGIXY_PREFIX }  else { Join-Path $env:LOCALAPPDATA "legixy" }),
  [switch]$NoModel
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ModelName = "paraphrase-multilingual-MiniLM-L12-v2"
$HfRepo    = "sentence-transformers/$ModelName"

function Info($m) { Write-Host "==> $m" -ForegroundColor Cyan }
function Fail($m) { Write-Host "error: $m" -ForegroundColor Red; exit 1 }

# ---- resolve release tag ----------------------------------------------------
$api = "https://api.github.com/repos/$Repo/releases"
if ($Version -eq "latest") {
  Info "resolving latest release of $Repo"
  $rel = Invoke-RestMethod -Uri "$api/latest" -Headers @{ "User-Agent" = "legixy-install" }
  $tag = $rel.tag_name
  if (-not $tag) { Fail "could not resolve latest release tag" }
} else {
  $tag = $Version
}
$asset = "legixy-$tag-x86_64-windows.zip"
$url   = "https://github.com/$Repo/releases/download/$tag/$asset"

# ---- download & unpack ------------------------------------------------------
$InstallDir = Join-Path $Prefix "share\legixy"
$BinDir     = Join-Path $Prefix "bin"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("legixy-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
  $zip = Join-Path $tmp $asset
  Info "downloading $asset ($tag)"
  Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing

  Info "installing into $InstallDir"
  if (Test-Path $InstallDir) { Remove-Item -Recurse -Force $InstallDir }
  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
  Expand-Archive -Path $zip -DestinationPath $tmp -Force
  # archive contains a single top-level legixy-<tag>-... directory
  $top = Get-ChildItem -Directory $tmp | Where-Object { $_.Name -like "legixy-*" } | Select-Object -First 1
  if (-not $top) { $top = Get-Item $tmp }
  Copy-Item -Path (Join-Path $top.FullName "*") -Destination $InstallDir -Recurse -Force

  # The archive ships bin\legixy.exe next to onnxruntime.dll; keep them together
  # (the exe loads the DLL from its own directory). We add that dir to PATH
  # rather than copying the exe away from its DLL.
  if (-not (Test-Path (Join-Path $BinDir "legixy.exe"))) {
    $found = Get-ChildItem -Path $InstallDir -Recurse -Filter "legixy.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { $BinDir = $found.Directory.FullName }
  }
} finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

# ---- PATH (user) ------------------------------------------------------------
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$BinDir*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$BinDir", "User")
  Info "added $BinDir to user PATH (restart your shell to pick it up)"
}

# ---- ONNX model -------------------------------------------------------------
if (-not $NoModel) {
  $dest = Join-Path $InstallDir "models\$ModelName"
  $base = "https://huggingface.co/$HfRepo/resolve/main"
  New-Item -ItemType Directory -Force -Path $dest | Out-Null
  Info "fetching embedding model: $HfRepo (~500 MB)"
  if (-not (Test-Path (Join-Path $dest "tokenizer.json"))) {
    Invoke-WebRequest -Uri "$base/tokenizer.json" -OutFile (Join-Path $dest "tokenizer.json") -UseBasicParsing
  }
  if (-not (Test-Path (Join-Path $dest "model.onnx"))) {
    try {
      Invoke-WebRequest -Uri "$base/onnx/model.onnx" -OutFile (Join-Path $dest "model.onnx") -UseBasicParsing
    } catch {
      Remove-Item (Join-Path $dest "model.onnx") -ErrorAction SilentlyContinue
      Fail "failed to fetch onnx/model.onnx. Export it with optimum (see docs/manual)."
    }
  }
  # The Windows binary resolves the model via LGX_MODELS_DIR (no shell wrapper).
  [Environment]::SetEnvironmentVariable("LGX_MODELS_DIR", $dest, "User")
  Info "model installed at $dest (LGX_MODELS_DIR set for your user)"
} else {
  Info "skipping model download (-NoModel). The semantic layer needs a model; see docs/manual."
}

Write-Host ""
Info "legixy $tag installed."
Write-Host "Next: open a NEW shell, run 'legixy init' in a project, then 'legixy check --formal'."
Write-Host "Full guide: docs\manual\manual.en.md"
