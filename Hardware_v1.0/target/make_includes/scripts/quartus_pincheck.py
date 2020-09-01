#!/usr/bin/python3

import sys

if len(sys.argv) == 1:
    sys.stdout.write("USAGE: %s filename\n" % sys.argv[0])
    sys.exit(0)

if len(sys.argv) != 2:
    sys.stderr.write("ERROR: invalid arguments.\n")
    sys.exit(1)

file = open(sys.argv[1], "rt")
check = False
num_checked = 0
num_failed = 0
num_auto = 0

for line in file.readlines():
    parts = line.split(";")
    if line.strip() == "":
        check = False
        continue
    if len(parts) > 3 and parts[1].strip() == "Name" and parts[2].strip() == "Pin #":
        for i in range(2, len(parts)):
            if parts[i].strip() == "Location assigned by":
                check = True
                check_field = i
                name = ""
        continue

    if check and len(parts) > check_field:
        prev_name = name
        name = parts[1].strip()
        pin = parts[2].strip()
        assigned_by = parts[check_field].strip()
        sys.stdout.write("%s,%s,%s\n" % (name, pin, assigned_by))
        num_checked += 1
        if assigned_by != "User":
            if name == prev_name + "(n)":
                sys.stdout.write("^ INFO: Negative pin of differential pair was automatically assigned.\n")
                num_auto += 1
            else:
                sys.stderr.write("^ ERROR: Pin %s (%s) is assigned by %s instead of User!\n" % (pin, name, assigned_by))
                num_failed += 1

file.close()

if num_checked == 0:
    sys.stderr.write("ERROR: No pins were checked!\n");
    sys.exit(1)

sys.stdout.write("\n")
sys.stdout.write("Number of pins checked: %d.\n" % num_checked)
sys.stdout.write("Number of pins failed : %d.\n" % num_failed)
sys.stdout.write("Number of (n) pins    : %d.\n" % num_auto)

if num_failed == 1:
    sys.stderr.write("ERROR: Not all pins are User assigned!\n");
    sys.exit(1)

sys.exit(0)
