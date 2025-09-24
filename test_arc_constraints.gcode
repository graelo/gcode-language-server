; Testing arc constraints
; Valid arc commands
G2 X10 Y10 I5 J5  ; Valid - has coordinates and center
G3 X20 Y20 R10     ; Valid - has coordinates and radius (Marlin only)
G2 I10 J10         ; Valid - complete circle with center

; Invalid arc commands
G2 R10 I5          ; Invalid - mixing R with I (Marlin)
G3 R15 J10         ; Invalid - mixing R with J (Marlin)
G2                 ; Invalid - no parameters at all (Prusa constraint)