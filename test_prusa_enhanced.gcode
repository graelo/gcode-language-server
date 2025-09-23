; Enhanced Prusa G-code test file
; Basic movement
G28 ; Home all axes
G29 ; Bed leveling
G90 ; Absolute positioning
G1 X50 Y50 Z0.2 F3000 ; Move to position

; Temperature control
M104 S210 ; Set hotend temperature
M140 S60 ; Set bed temperature
M109 S210 ; Wait for hotend
M190 S60 ; Wait for bed

; Advanced Prusa features
M862.3 P "MINI" ; Check model name
M593 F45 X1 ; Set input shaper for X axis
M900 K0.05 ; Set linear advance
G64 D0.1 ; Measure Z height with offset

; MMU3 commands
T0 ; Select tool 0
M701 P0 ; Load filament to slot 0
M600 T0 ; Filament change on tool 0

; Print progress
M73 P50 R30 ; Set progress to 50%, 30min remaining

; Fan control
M106 S128 ; Set fan to 50%
M107 ; Turn off fan

; End sequence
M84 ; Disable steppers
