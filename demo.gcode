; Test G-code file to demonstrate the fixes
; ===========================================

; These should show ERRORS (G0/G1 without coordinate parameters):
G0
G1

; These should be VALID (G0/G1 with coordinate parameters):
G0 X10
G1 Y20
G0 Z5.5
G1 X10 Y20 Z0.3 E2.5 F1500

; Parameter completion test:
; Type "G0 " and trigger completion - should show X, Y, Z, F parameters
; Type "G1 " and trigger completion - should show X, Y, Z, E, F parameters

; Other commands (should work as before):
G28    ; Home all axes
G90    ; Absolute positioning
M104 S200   ; Set hotend temp