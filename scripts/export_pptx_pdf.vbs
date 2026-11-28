Dim args, inPath, outPath, ppApp, pres
Set args = WScript.Arguments
If args.Count < 2 Then
  WScript.Echo "Usage: cscript //nologo export_pptx_pdf.vbs <in.pptx> <out.pdf>"
  WScript.Quit 2
End If
inPath = args(0)
outPath = args(1)

Set ppApp = CreateObject("PowerPoint.Application")
Set pres = ppApp.Presentations.Open(inPath, True, False, False)
WScript.Echo "PowerPoint slide count: " & pres.Slides.Count
pres.SaveAs outPath, 32
pres.Close
ppApp.Quit
WScript.Echo "Exported PPTX PDF: " & outPath
