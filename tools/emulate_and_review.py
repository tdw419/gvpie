#!/usr/bin/env python3
import json
import re
import sys

class Emulator:
    def __init__(self):
        self.memory = bytearray(65536)
        self.symbols = {}

    def apply_operations(self, operations):
        for op in operations:
            op_type = op["type"]
            if op_type == "ADD_SYMBOL":
                self.symbols[op["name"]] = op["addr"]
            elif op_type == "WRITE_BYTE":
                addr = op["addr"]
                value = op["value"]
                if 0 <= addr < len(self.memory):
                    self.memory[addr] = value
            elif op_type == "WRITE_BLOCK":
                addr = op["addr"]
                data = op["data"]
                if 0 <= addr < len(self.memory) and 0 <= addr + len(data) <= len(self.memory):
                    self.memory[addr:addr + len(data)] = bytearray(data)

    def dump_screen(self):
        screen = ""
        for y in range(25):
            line = ""
            for x in range(80):
                char_addr = 0xA000 + y * 80 * 2 + x * 2
                char_code = self.memory[char_addr]
                if 32 <= char_code < 127:
                    line += chr(char_code)
                else:
                    line += ' ' # Replace non-printable characters
            screen += line + "\n"
        return screen

def parse_operations(file_content):
    operations = []
    in_operations_block = False
    for line in file_content.splitlines():
        line = line.strip()
        if line == "OPERATIONS:":
            in_operations_block = True
            continue
        if not in_operations_block or not line or line.startswith("COMMENT"):
            continue

        parts = line.split()
        op_type = parts[0]

        if op_type == "ADD_SYMBOL":
            operations.append({"type": op_type, "name": parts[1], "addr": int(parts[2], 16)})
        elif op_type == "WRITE_BYTE":
            operations.append({"type": op_type, "addr": int(parts[1], 16), "value": int(parts[2], 16)})
        elif op_type == "WRITE_BLOCK":
            addr = int(parts[2], 16)
            # This regex finds all hex values in the rest of the string
            data_str = " ".join(parts[3:])
            data = [int(b, 16) for b in re.findall(r'0x[0-9a-fA-F]+', data_str)]
            operations.append({"type": op_type, "name": parts[1], "addr": addr, "data": data})
        else:
             operations.append({"type": op_type, "parts": parts[1:]})

    return operations

def validate_operations(operations):
    allowed_ops = ["ADD_SYMBOL", "WRITE_BYTE", "WRITE_BLOCK"]
    for op in operations:
        if op["type"] not in allowed_ops:
            return False, f"Illegal operation: {op['type']}"
    return True, None

def main(in_file, out_file):
    with open(in_file, 'r') as f:
        content = f.read()

    operations = parse_operations(content)
    is_valid, error = validate_operations(operations)

    if not is_valid:
        review = {"success": False, "issues": [error]}
    else:
        emu = Emulator()
        emu.apply_operations(operations)
        screen_dump = emu.dump_screen()
        review = {
            "success": True,
            "issues": [],
            "screen_dump": screen_dump,
        }

    with open(out_file, 'w') as f:
        json.dump(review, f, indent=2)
    print(f"Review file generated at {out_file}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: emulate_and_review.py <iteration_file> <review_file>")
        sys.exit(1)

    main(sys.argv[1], sys.argv[2])
