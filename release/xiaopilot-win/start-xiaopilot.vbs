Set WshShell = CreateObject("WScript.Shell")
' 切换工作目录
WshShell.CurrentDirectory = "C:\Users\Family\xiaopilot-win"
' 后台隐藏运行程序，0=隐藏窗口
WshShell.Run "xiaopilot-win.exe", 0, False
Set WshShell = Nothing
WScript.Quit