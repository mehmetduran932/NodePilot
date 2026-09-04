Add-Type -AssemblyName System.Drawing
$iconDir = "C:\dev\NodePilot\apps\desktop\src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null

$bmp = New-Object System.Drawing.Bitmap 32, 32
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(59, 130, 246))
$pen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2)
$g.DrawEllipse($pen, 6, 6, 20, 20)
$brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$g.FillEllipse($brush, 13, 13, 6, 6)
$g.Dispose()

$bmp.Save("$iconDir\32x32.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Save("$iconDir\128x128.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Save("$iconDir\128x128@2x.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Save("$iconDir\icon.png", [System.Drawing.Imaging.ImageFormat]::Png)

$hIcon = $bmp.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hIcon)
$fs = New-Object System.IO.FileStream("$iconDir\icon.ico", [System.IO.FileMode]::Create)
$icon.Save($fs)
$fs.Close()
$icon.Dispose()
$bmp.Dispose()
Write-Output "Icons generated successfully"
