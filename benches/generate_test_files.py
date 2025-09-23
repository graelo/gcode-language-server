#!/usr/bin/env python3
"""
Generate realistic G-code test files for benchmarking
"""

import random
import math


def generate_large_print_file(filename, layers=100, moves_per_layer=200):
    """Generate a realistic 3D print G-code file"""
    with open(filename, "w") as f:
        # Header
        f.write("; Generated test file for benchmarking\n")
        f.write("; Simulates a typical 3D print with {} layers\n".format(layers))
        f.write("; PrusaSlicer compatible\n")
        f.write(";\n")

        # Start sequence
        f.write("M115 U3.9.0 ; tell printer latest fw version\n")
        f.write("M83  ; extruder relative mode\n")
        f.write("M104 S215 ; set extruder temp\n")
        f.write("M140 S60 ; set bed temp\n")
        f.write("M190 S60 ; wait for bed temp\n")
        f.write("M109 S215 ; wait for extruder temp\n")
        f.write("G28 W ; home all without mesh bed level\n")
        f.write("G80 ; mesh bed leveling\n")
        f.write("G1 Y-3.0 F1000.0 ; go outside print area\n")
        f.write("G92 E0.0 ; reset extrusion distance\n")
        f.write("G1 X60.0 E9.0 F1000.0 ; intro line\n")
        f.write("G1 X100.0 E12.5 F1000.0 ; intro line\n")
        f.write("G92 E0.0 ; reset extrusion distance\n")
        f.write("\n")

        # Print layers
        for layer in range(layers):
            layer_height = layer * 0.2
            f.write("; LAYER:{}\n".format(layer))
            f.write("G1 Z{:.3f} F720 ; move to layer height\n".format(layer_height))

            # Generate moves for this layer
            center_x, center_y = 100, 100
            radius = 30 + layer * 0.1
            e_position = 0

            for move in range(moves_per_layer):
                angle = (move / moves_per_layer) * 2 * math.pi
                x = center_x + radius * math.cos(angle) + random.uniform(-2, 2)
                y = center_y + radius * math.sin(angle) + random.uniform(-2, 2)

                # Vary feed rate and extrusion
                feed_rate = random.randint(1200, 2400)
                e_distance = random.uniform(0.02, 0.08)
                e_position += e_distance

                f.write(
                    "G1 X{:.3f} Y{:.3f} E{:.5f} F{}\n".format(
                        x, y, e_distance, feed_rate
                    )
                )

                # Occasionally add temperature or fan commands
                if move % 50 == 0:
                    temp_variation = random.randint(-5, 5)
                    f.write("M104 S{}\n".format(215 + temp_variation))

                if move % 75 == 0:
                    fan_speed = random.randint(0, 255)
                    f.write("M106 S{}\n".format(fan_speed))

        # End sequence
        f.write("\n; End sequence\n")
        f.write("G4 ; wait\n")
        f.write("M221 S100 ; reset flow\n")
        f.write("M900 K0 ; reset LA\n")
        f.write("M104 S0 ; turn off temperature\n")
        f.write("M140 S0 ; turn off heatbed\n")
        f.write("M107 ; turn off fan\n")
        f.write("G1 X0 Y200; home X axis and push Y forward\n")
        f.write("M84 ; disable motors\n")


def generate_complex_cnc_file(filename, operations=50):
    """Generate a complex CNC G-code file with various operations"""
    with open(filename, "w") as f:
        f.write("; Complex CNC operations test file\n")
        f.write(";\n")

        # Setup
        f.write("G90 ; Absolute positioning\n")
        f.write("G94 ; Feed rate per minute\n")
        f.write("G17 ; XY plane selection\n")
        f.write("M3 S1000 ; Start spindle\n")
        f.write("G0 Z5 ; Safe height\n")
        f.write("\n")

        for op in range(operations):
            # Various CNC operations
            x = random.uniform(0, 100)
            y = random.uniform(0, 100)
            z = random.uniform(-5, 0)

            if op % 10 == 0:
                # Drill cycle
                f.write("G98 G81 X{:.3f} Y{:.3f} Z{:.3f} R2 F300\n".format(x, y, z))
                f.write("G80 ; Cancel drill cycle\n")
            elif op % 7 == 0:
                # Circular interpolation
                i = random.uniform(-5, 5)
                j = random.uniform(-5, 5)
                f.write("G2 X{:.3f} Y{:.3f} I{:.3f} J{:.3f} F1000\n".format(x, y, i, j))
            elif op % 5 == 0:
                # Tool change
                tool_num = random.randint(1, 10)
                f.write("M6 T{}\n".format(tool_num))
                f.write("M3 S{}\n".format(random.randint(800, 1200)))
            else:
                # Linear move
                feed = random.randint(200, 1500)
                f.write("G1 X{:.3f} Y{:.3f} Z{:.3f} F{}\n".format(x, y, z, feed))

        # End
        f.write("\nM5 ; Stop spindle\n")
        f.write("G0 Z25 ; Safe height\n")
        f.write("M30 ; Program end\n")


def generate_error_heavy_file(filename, lines=1000):
    """Generate a file with many different types of errors"""
    with open(filename, "w") as f:
        f.write("; Error-heavy test file for validation benchmarking\n")
        f.write(";\n")

        for i in range(lines):
            error_type = i % 12

            if error_type == 0:
                f.write("G1\n")  # Missing required parameters
            elif error_type == 1:
                f.write("G0\n")  # Missing required parameters
            elif error_type == 2:
                f.write("G999 X10\n")  # Unknown command
            elif error_type == 3:
                f.write("M999 S100\n")  # Unknown command
            elif error_type == 4:
                f.write("G1 X Y10\n")  # Missing parameter value
            elif error_type == 5:
                f.write("M104 S\n")  # Missing parameter value
            elif error_type == 6:
                f.write("G1 Xinvalid Y10\n")  # Invalid parameter value
            elif error_type == 7:
                f.write("M104 Sinvalid\n")  # Invalid parameter value
            elif error_type == 8:
                f.write("INVALID_COMMAND\n")  # Completely invalid
            elif error_type == 9:
                f.write("G1 X10 Q15\n")  # Invalid parameter for command
            elif error_type == 10:
                f.write("123INVALID\n")  # Invalid syntax
            else:
                # Some valid commands mixed in
                f.write("G1 X{:.3f} Y{:.3f} F1500\n".format(i * 0.1, i * 0.1))


if __name__ == "__main__":
    import os

    # Create test files directory
    os.makedirs("benches/test_files", exist_ok=True)

    print("Generating large print file (20,000+ lines)...")
    generate_large_print_file(
        "benches/test_files/large_print.gcode", layers=100, moves_per_layer=200
    )

    print("Generating very large print file (50,000+ lines)...")
    generate_large_print_file(
        "benches/test_files/very_large_print.gcode", layers=200, moves_per_layer=250
    )

    print("Generating complex CNC file...")
    generate_complex_cnc_file("benches/test_files/complex_cnc.gcode", operations=500)

    print("Generating error-heavy file...")
    generate_error_heavy_file("benches/test_files/error_heavy.gcode", lines=5000)

    print("Test files generated successfully!")
