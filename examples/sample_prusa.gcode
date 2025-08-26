; vim: gcode_flavor=prusa
; This is a sample G-code file with a Prusa flavor modeline
; The language server should detect the modeline and use Prusa commands

G28             ; Home all axes
G1 Z15.0 F9000  ; Move the platform down 15mm
G92 E0          ; Set extruder position to zero
G1 F200 E3      ; Extrude 3mm of feed stock
G92 E0          ; Set extruder position to zero again

; Start printing
G1 F1500        ; Set feedrate
G1 X2.0 Y2.0 Z0.3 F3000.0 ; Go to front left corner
G1 X2.0 Y200.0 Z0.3 F1500.0 E15 ; Draw the first line
G1 X2.3 Y200.0 Z0.3 F5000.0 ; Move to side a little  
G1 X2.3 Y2 Z0.3 F1500.0 E30 ; Draw the second line

; Prusa-specific command
M250 S1000      ; Set LCD contrast (Prusa-specific)
