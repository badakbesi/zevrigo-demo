@echo off
rem peserta-demo-zevrigo.cmd - paket peserta jaringan demo Zevrigo (regtest), Windows x64.
rem
rem Menyalakan node regtest di jendela sendiri, tersambung sendiri ke seed resmi jaringan demo (--seed-resmi), membuat
rem wallet bila belum ada, lalu menambang satu blok setiap ZVRG_JEDA detik (bawaan 120, sama dengan waktu blok target)
rem dengan hadiah ke alamat baru wallet itu. Hentikan miner dengan Ctrl-C di jendela ini; hentikan node dengan Ctrl-C di
rem jendela node.
rem
rem   peserta-demo-zevrigo.cmd
rem   set ZVRG_JEDA=300 lalu peserta-demo-zevrigo.cmd       jeda lain (detik)
rem   set ZVRG_DATA=D:\folder lalu peserta-demo-zevrigo.cmd  folder data lain (bawaan %USERPROFILE%\.zevrigo)
rem
rem Koin jaringan demo tidak bernilai. Node hanya membuka RPC di 127.0.0.1 dan tidak menerima sambungan masuk dari luar.
setlocal EnableExtensions
set "DIR=%~dp0"
if not defined ZVRG_DATA set "ZVRG_DATA=%USERPROFILE%\.zevrigo"
if not defined ZVRG_JEDA set "ZVRG_JEDA=120"
set "NODE=%DIR%zevrigo-node.exe"
set "CLI=%DIR%zevrigo-cli.exe"
set "MINER=%DIR%zevrigo-miner.exe"
for %%b in ("%NODE%" "%CLI%" "%MINER%") do if not exist "%%~b" (
  echo Program tidak ditemukan: %%~b. Jalankan dari folder paket yang sudah diekstrak.
  goto gagal
)
echo %ZVRG_JEDA%| findstr /r /x "[0-9][0-9]*" >nul || (
  echo ZVRG_JEDA harus jumlah detik, misalnya 120
  goto gagal
)
if not exist "%ZVRG_DATA%" mkdir "%ZVRG_DATA%" || goto gagal

echo == Zevrigo jaringan demo (regtest) ^| folder data: %ZVRG_DATA%
call :tinggi
if defined H (
  echo Node sudah berjalan di 127.0.0.1:28801; dipakai apa adanya.
) else (
  echo Menyalakan node di jendela baru. Biarkan jendela itu terbuka; tutup dengan Ctrl-C bila selesai.
  start "Zevrigo node (Ctrl-C untuk berhenti)" "%NODE%" --regtest --datadir "%ZVRG_DATA%" --seed-resmi
  for /l %%i in (1,1,30) do (
    call :tinggi
    if not defined H timeout /t 2 /nobreak >nul
  )
)
call :tinggi
if not defined H (
  echo Node tidak menyala. Lihat pesan di jendela node.
  goto gagal
)

echo Menunggu node tersambung ke jaringan demo dan mengejar chain...
set /a N=0
set /a STABIL=0
set "LALU=-1"
:tunggu
call :tinggi
call :publik
if not defined H set "H=0"
set "PUB=1"
if not defined P (
  set "PUB=0"
  set "P=0"
)
if "%PUB%"=="1" if %P% GTR 0 if %H% GEQ %P% goto sinkron
if "%PUB%"=="0" if "%H%"=="%LALU%" (set /a STABIL+=1) else (set /a STABIL=0)
if %H% GTR 0 if %STABIL% GEQ 4 goto sinkron
set "LALU=%H%"
set /a N+=1
if %N% GEQ 120 (
  echo Node belum mengejar chain dalam 10 menit ^(tinggi lokal %H%, publik %P%^). Periksa internet, lalu coba lagi.
  goto gagal
)
timeout /t 5 /nobreak >nul
goto tunggu
:sinkron
echo Node sinkron: tinggi %H%.

if not exist "%ZVRG_DATA%\regtest\wallet.json" (
  echo.
  echo == Wallet baru
  echo Wallet belum ada. Buat password ^(minimal 8 karakter^), lalu CATAT 24 kata yang tampil di kertas dan simpan di tempat aman.
  echo Tanpa 24 kata itu wallet tidak bisa dipulihkan.
  "%CLI%" --datadir "%ZVRG_DATA%" wallet-create || goto gagal
  pause
)
set "ALAMAT="
set "PAYOUT="
for /f "tokens=1,2" %%a in ('^""%CLI%" --datadir "%ZVRG_DATA%" address^"') do (
  if "%%a"=="alamat:" set "ALAMAT=%%b"
  if "%%a"=="payload:" set "PAYOUT=%%b"
)
if not defined PAYOUT (
  echo Alamat wallet gagal dibaca.
  goto gagal
)
echo.
echo == Menambang: satu blok setiap %ZVRG_JEDA% detik, hadiah ke %ALAMAT%
echo Lihat saldo: "%CLI%" --datadir "%ZVRG_DATA%" balance
echo Hentikan dengan Ctrl-C.
:tambang
"%MINER%" --payout %PAYOUT% --blocks 1 || echo Putaran gagal, dicoba lagi sesudah jeda.
timeout /t %ZVRG_JEDA% /nobreak >nul
goto tambang

:tinggi
set "H="
for /f "tokens=2" %%h in ('^""%CLI%" info 2^>nul ^| findstr /b /c:"tinggi:"^"') do set "H=%%h"
exit /b 0

:publik
set "P="
for /f %%h in ('powershell -NoProfile -Command "try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (Invoke-RestMethod -Method Post -Uri 'https://demo-gerbang-rpc.zevrigo.online' -ContentType 'application/json' -TimeoutSec 20 -Body (@{jsonrpc='2.0';id=1;method='get_chain_info';params=@{}} | ConvertTo-Json)).result.height } catch {}" 2^>nul') do set "P=%%h"
exit /b 0

:gagal
echo.
pause
exit /b 1
