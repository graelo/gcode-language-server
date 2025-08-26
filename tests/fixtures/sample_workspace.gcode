; gcode_flavor=workspace_test
; This file uses a workspace-specific flavor
; The language server should use the workspace_test flavor for this file

G0 X10 Y10      ; Rapid move (workspace version)
G1 X50 Y50 F1000 ; Linear move
G99             ; Custom workspace command
G28             ; Home (should fall back to standard behavior)
