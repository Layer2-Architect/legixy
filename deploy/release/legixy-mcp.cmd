@echo off
rem Release MCP launcher (Windows). Lives in bin\ next to legixy.exe / onnxruntime.dll.
set "LGX_BIN=%~dp0legixy.exe"
node "%~dp0..\ts-mcp\dist\index.js" %*
