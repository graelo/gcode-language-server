; Test G-code file for language server testing
G28 ; Home all axes
M104 S210 ; Set hotend temperature
G1 X10 Y20 Z0.3 F1500 ; Move to position
G0 X0 Y0 ; Rapid move to origin
GUNKNOWN X10 ; This should show as unknown command
G1 ; This should show missing required parameters