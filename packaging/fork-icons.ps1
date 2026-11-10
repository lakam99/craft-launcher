# Original download-arrow icon; MIT OR Apache-2.0. No third-party artwork or font is used.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$iconRoot = Join-Path $root 'assets/app-icon'
$svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><rect x="16" y="16" width="224" height="224" fill="#182332"/><polygon points="112,56 144,56 144,120 176,120 128,168 80,120 112,120" fill="#65ddcf"/><path d="M72 176V200H184V176" fill="none" stroke="#749cff" stroke-width="16"/></svg>'
$pngs = @{}
foreach ($size in @(16,20,24,32,40,48,64,128,256,512,1024)) {
  $bitmap = [Drawing.Bitmap]::new($size,$size)
  $graphics = [Drawing.Graphics]::FromImage($bitmap)
  $background = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#182332'))
  $arrow = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#65ddcf'))
  $tray = [Drawing.Pen]::new([Drawing.ColorTranslator]::FromHtml('#749cff'),16)
  try {
    $graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.ScaleTransform(($size/256.0),($size/256.0))
    $graphics.FillRectangle($background,16,16,224,224)
    $points = [Drawing.PointF[]]@([Drawing.PointF]::new(112,56),[Drawing.PointF]::new(144,56),[Drawing.PointF]::new(144,120),[Drawing.PointF]::new(176,120),[Drawing.PointF]::new(128,168),[Drawing.PointF]::new(80,120),[Drawing.PointF]::new(112,120))
    $graphics.FillPolygon($arrow,$points)
    $graphics.DrawLines($tray,[Drawing.PointF[]]@([Drawing.PointF]::new(72,176),[Drawing.PointF]::new(72,200),[Drawing.PointF]::new(184,200),[Drawing.PointF]::new(184,176)))
    $stream = [IO.MemoryStream]::new()
    try { $bitmap.Save($stream,[Drawing.Imaging.ImageFormat]::Png); $pngs[$size]=$stream.ToArray() } finally { $stream.Dispose() }
  } finally { $tray.Dispose(); $arrow.Dispose(); $background.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }
}
foreach ($size in @(16,24,32,48,64,128,256,512)) {
  $directory=Join-Path $iconRoot "hicolor/${size}x${size}/apps"
  [IO.Directory]::CreateDirectory($directory) | Out-Null
  [IO.File]::WriteAllBytes((Join-Path $directory 'ai.storyteller.artcraft-launcher.png'),$pngs[$size])
}
[IO.File]::WriteAllBytes((Join-Path $iconRoot 'artcraft-launcher-1024.png'),$pngs[1024])
foreach($file in @('artcraft-launcher.svg','hicolor/scalable/apps/ai.storyteller.artcraft-launcher.svg')) { [IO.File]::WriteAllText((Join-Path $iconRoot $file),$svg,[Text.UTF8Encoding]::new($false)) }
$iconStream=[IO.File]::Create((Join-Path $iconRoot 'artcraft-launcher.ico'))
$writer=[IO.BinaryWriter]::new($iconStream)
try {
  $sizes=@(16,20,24,32,40,48,64,128,256)
  $writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]$sizes.Count)
  $offset=6+16*$sizes.Count
  foreach($size in $sizes){$dimension=if($size -eq 256){0}else{$size};$writer.Write([byte]$dimension);$writer.Write([byte]$dimension);$writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]32);$writer.Write([uint32]$pngs[$size].Length);$writer.Write([uint32]$offset);$offset+=$pngs[$size].Length}
  foreach($size in $sizes){$writer.Write([byte[]]$pngs[$size])}
} finally {$writer.Dispose()}
$icns=[IO.MemoryStream]::new()
try {
  $chunks=@(@(16,'icp4'),@(32,'icp5'),@(32,'ic11'),@(64,'icp6'),@(64,'ic12'),@(128,'ic07'),@(256,'ic08'),@(256,'ic13'),@(512,'ic09'),@(512,'ic14'),@(1024,'ic10'))
  foreach($chunk in $chunks){$data=$pngs[$chunk[0]];$type=[Text.Encoding]::ASCII.GetBytes($chunk[1]);$icns.Write($type);$length=[BitConverter]::GetBytes([uint32]($data.Length+8));[Array]::Reverse($length);$icns.Write($length);$icns.Write($data)}
  $file=[IO.File]::Create((Join-Path $iconRoot 'artcraft-launcher.icns'))
  try {$file.Write([Text.Encoding]::ASCII.GetBytes('icns'));$length=[BitConverter]::GetBytes([uint32]($icns.Length+8));[Array]::Reverse($length);$file.Write($length);$file.Write($icns.ToArray())}finally{$file.Dispose()}
}finally{$icns.Dispose()}
$template=[Drawing.Bitmap]::new(44,44)
$templateGraphics=[Drawing.Graphics]::FromImage($template)
$black=[Drawing.SolidBrush]::new([Drawing.Color]::Black)
$blackTray=[Drawing.Pen]::new([Drawing.Color]::Black,16)
try {
  $templateGraphics.SmoothingMode=[Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $templateGraphics.ScaleTransform((44/256.0),(44/256.0))
  $templateGraphics.FillPolygon($black,[Drawing.PointF[]]@([Drawing.PointF]::new(112,56),[Drawing.PointF]::new(144,56),[Drawing.PointF]::new(144,120),[Drawing.PointF]::new(176,120),[Drawing.PointF]::new(128,168),[Drawing.PointF]::new(80,120),[Drawing.PointF]::new(112,120)))
  $templateGraphics.DrawLines($blackTray,[Drawing.PointF[]]@([Drawing.PointF]::new(72,176),[Drawing.PointF]::new(72,200),[Drawing.PointF]::new(184,200),[Drawing.PointF]::new(184,176)))
  $template.Save((Join-Path $iconRoot 'tray-template.png'),[Drawing.Imaging.ImageFormat]::Png)
}finally{$blackTray.Dispose();$black.Dispose();$templateGraphics.Dispose();$template.Dispose()}
$backgroundPath=Join-Path $root 'packaging/macos/dmg'
[IO.File]::WriteAllText((Join-Path $backgroundPath 'background.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="660" height="400"><rect width="660" height="400" fill="#f2f4f7"/><rect width="232" height="400" fill="#182332"/></svg>',[Text.UTF8Encoding]::new($false))
$background=[Drawing.Bitmap]::new(660,400)
$drawing=[Drawing.Graphics]::FromImage($background)
$brush=[Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#182332'))
try{$drawing.Clear([Drawing.ColorTranslator]::FromHtml('#f2f4f7'));$drawing.FillRectangle($brush,0,0,232,400);$background.Save((Join-Path $backgroundPath 'background.tiff'),[Drawing.Imaging.ImageFormat]::Tiff)}finally{$brush.Dispose();$drawing.Dispose();$background.Dispose()}
Write-Output 'Generated original fork SVG, PNG, ICO, ICNS and disk-image background.'
