Dim args, inPath, outPath, word, doc
Set args = WScript.Arguments
If args.Count < 2 Then
  WScript.Echo "Usage: cscript //nologo export_word_pdf.vbs <in.docx> <out.pdf>"
  WScript.Quit 2
End If
inPath = args(0)
outPath = args(1)

Set word = CreateObject("Word.Application")
word.Visible = False
word.DisplayAlerts = 0
Set doc = word.Documents.Open(inPath, False, True)
doc.Repaginate
WScript.Echo "Word page count: " & doc.ComputeStatistics(2)
' wdExportFormatPDF = 17, wdExportOptimizeForPrint = 0, wdExportAllDocument = 0,
' wdExportDocumentContent = 0, wdExportCreateHeadingBookmarks = 1
doc.ExportAsFixedFormat outPath, 17, False, 0, 0, 1, 1, 0, True, True, 1, True, True, False
doc.Close False
word.Quit
WScript.Echo "Exported PDF: " & outPath
