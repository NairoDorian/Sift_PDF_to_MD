@echo off
rem Build Sift with the layout model on the GPU (candle's CUDA backend, feature sift-core/cuda).
rem Needs the CUDA toolkit (13.4 or older: cudarc 0.19 does not know 13.5 yet) and MSVC's cl.exe
rem (nvcc compiles candle's kernels with it). The default build needs neither.
setlocal
if "%SIFT_CUDA%"=="" set "SIFT_CUDA=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
rem `call` in front: cmd strips the quotes of a command that starts with a quoted path holding
rem "(x86)", and vswhere then "is not recognized"
for /f "usebackq tokens=*" %%i in (`call "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -property installationPath`) do set "VSDIR=%%i"
if not defined VSDIR (echo Visual Studio with C++ was not found ^(vswhere^) & exit /b 1)
call "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
set "CUDA_PATH=%SIFT_CUDA%"
rem CUDA 13 headers need the conforming preprocessor; cl.exe reads extra options from CL
set "CL=/Zc:preprocessor"
rem kernels for this GPU only (compute capability "8.9" -> 89)
if "%CUDA_COMPUTE_CAP%"=="" for /f "usebackq tokens=1,2 delims=." %%a in (`nvidia-smi --query-gpu^=compute_cap --format^=csv^,noheader`) do set "CUDA_COMPUTE_CAP=%%a%%b"
set "PATH=%SIFT_CUDA%\bin;%PATH%"
cd /d "%~dp0.."
cargo build --release -p sift-cli --features sift-core/cuda --target-dir target-cuda %*
