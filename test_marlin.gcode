; Test G-code for Marlin flavor
G28 ; Home all axes
M104 S210 ; Set hotend temperature  
M140 S60 ; Set bed temperature
M190 S60 ; Wait for bed temperature
M109 S210 ; Wait for hotend temperature

G29 ; Auto bed leveling
M851 Z-0.1 ; Set Z probe offset

; Movement with linear advance
M900 K0.3 ; Set linear advance K factor
G1 X10 Y10 F1500 ; Move to position
G1 X20 Y20 E0.5 F800 ; Print move with extrusion

; Motion control settings
M201 X3000 Y3000 Z100 E5000 ; Set max acceleration
M203 X300 Y300 Z20 E50 ; Set max feedrate
M204 S1000 T3000 ; Set acceleration
M205 X8 Y8 Z0.3 E5 ; Set jerk

G0 X0 Y0 ; Return to origin
M107 ; Turn off fan
M104 S0 ; Turn off hotend
M140 S0 ; Turn off bed