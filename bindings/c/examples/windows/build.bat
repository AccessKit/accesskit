@echo off
cl /nologo /W3 /DUNICODE /D_UNICODE /I..\..\include /c hello_world.c
Rem This script assumes you are building this example from the downloaded package on a 64bit machine.
link /NOLOGO /SUBSYSTEM:CONSOLE hello_world.obj ..\..\lib\windows\x86_64\msvc\static\accesskit.lib advapi32.lib bcrypt.lib kernel32.lib ole32.lib oleaut32.lib shell32.lib uiautomationcore.lib user32.lib userenv.lib winspool.lib ws2_32.lib
