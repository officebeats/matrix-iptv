@echo off
taskkill /F /IM matrix-iptv.exe 2>nul
cd /d "c:\Users\admin-beats\Documents\00 Vibe Coding\IPTV"
start "" ".\target\release\matrix-iptv.exe"
