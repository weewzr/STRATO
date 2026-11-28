<#
  Export .docx to PDF via Microsoft Word 16 COM automation and render sample PNG pages.
#>
param(
  [Parameter(Mandatory = $true)][string]$Docx,
  [Parameter(Mandatory = $true)][string]$Pdf,
  [string]$PngDir = '',
  [string]$Pages = '',
  [int]$Dpi = 120
)
$ErrorActionPreference = 'Stop'
$Docx = (Resolve-Path -LiteralPath $Docx).Path
$Pdf = [IO.Path]::GetFullPath($Pdf)
$before = @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
$word = $null
$doc = $null
try {
  $word = New-Object -ComObject Word.Application
  $word.Visible = $false
  $word.DisplayAlerts = 0
  $doc = $word.Documents.Open($Docx, $false, $true)
  $pageCount = $doc.ComputeStatistics(2)
  if (Test-Path -LiteralPath $Pdf) { Remove-Item -LiteralPath $Pdf -Force }
  $doc.ExportAsFixedFormat($Pdf, 17, $false, 0, 0, 1, 1, 0, $true, $true, 1, $true, $true, $false)
  Write-Output ("Word: {0} pages; PDF written to {1}" -f $pageCount, $Pdf)
  $pageNums = @(); if ($Pages -ne '') { $pageNums = @($Pages -split ',' | ForEach-Object { [int]$_.Trim() }) }
  if ($PngDir -ne '' -and $pageNums.Count -gt 0) {
    Add-Type -AssemblyName System.Drawing
    New-Item -ItemType Directory -Force -Path $PngDir | Out-Null
    $win = $word.ActiveWindow
    $win.View.Type = 3
    foreach ($p in $pageNums) {
      if ($p -gt $pageCount -or $p -lt 1) { continue }
      $pg = $win.ActivePane.Pages.Item($p)
      $bytes = [byte[]]$pg.EnhMetaFileBits
      $ms = New-Object IO.MemoryStream(, $bytes)
      $emf = New-Object System.Drawing.Imaging.Metafile($ms)
      $w = [int](8.5 * $Dpi); $h = [int](11 * $Dpi)
      $bmp = New-Object System.Drawing.Bitmap($w, $h)
      $bmp.SetResolution($Dpi, $Dpi)
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      $g.Clear([System.Drawing.Color]::White)
      $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
      $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
      $g.DrawImage($emf, 0, 0, $w, $h)
      $g.Dispose()
      $out = Join-Path $PngDir ("page{0:D3}.png" -f $p)
      $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
      $bmp.Dispose(); $emf.Dispose(); $ms.Dispose()
      Write-Output ("page image: {0}" -f $out)
    }
  }
}
finally {
  if ($doc -ne $null) { try { $doc.Close($false) } catch {} }
  if ($word -ne $null) { try { $word.Quit() } catch {} }
  Start-Sleep -Milliseconds 500
  foreach ($proc in @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue)) {
    if ($before -notcontains $proc.Id) { try { $proc.Kill() } catch {} }
  }
}
