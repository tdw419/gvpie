#!/usr/bin/env python3
from PIL import Image

def U24_to_rgb(val):
    r = (val >> 16) & 0xFF
    g = (val >> 8) & 0xFF
    b = val & 0xFF
    return r, g, b

class Assembler:
    def __init__(self, width=80, height=50):
        self.img = Image.new('RGB', (width, height), 'black')
        self.code_x = 0
        self.code_y = 1
        self.width = width
        self.labels = {}
        self.unresolved = []

    def write_op(self, op, rd=0, rs=0, b=0):
        if self.code_x >= self.width:
            self.code_x = 0
            self.code_y += 1
        r = op & 0xF
        g = (rd << 4) | rs
        self.img.putpixel((self.code_x, self.code_y), (r, g, b))
        self.code_x += 1

    def write_imm(self, val):
        if self.code_x >= self.width:
            self.code_x = 0
            self.code_y += 1
        self.img.putpixel((self.code_x, self.code_y), U24_to_rgb(val))
        self.code_x += 1

    def assemble(self, in_file, out_file):
        with open(in_file, 'r') as f:
            lines = f.readlines()

        # First pass: collect labels
        ip = 0
        for line in lines:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if ':' in line:
                label = line.split(':')[0]
                self.labels[label] = ip
            else:
                ip += self.get_instruction_size(line)

        # Second pass: generate code
        ip = 0
        for line in lines:
            line = line.strip()
            if not line or line.startswith('#') or ':' in line:
                continue

            parts = line.split()
            mnemonic = parts[0].upper()

            current_ip = ip
            ip += self.get_instruction_size(line)

            if mnemonic == 'MOV':
                reg = int(parts[1][1])
                val_str = parts[2]
                if val_str.startswith("'"): # Character literal
                    val = ord(val_str[1])
                else:
                    val = int(val_str, 0)
                self.write_op(0x1, rd=reg)
                self.write_imm(val)
            elif mnemonic == 'OUT':
                reg = int(parts[1][1])
                port = int(parts[2], 0)
                self.write_op(0xC, rs=reg)
                self.write_imm(port)
            elif mnemonic == 'ADD':
                rd = int(parts[1][1])
                rs = int(parts[2][1])
                self.write_op(0x3, rd=rd, rs=rs)
            elif mnemonic == 'SUB':
                rd = int(parts[1][1])
                rs = int(parts[2][1])
                self.write_op(0x4, rd=rd, rs=rs)
            elif mnemonic == 'CMP':
                ra = int(parts[1][1])
                val_str = parts[2]
                if val_str.startswith('R'):
                    rb = int(val_str[1])
                    self.write_op(0x5, rd=ra, rs=rb)
                else:
                    # This is a CMP with an immediate value, which is not in the spec.
                    # We can emulate it with MOV and CMP.
                    # MOV R7, imm
                    # CMP Ra, R7
                    val = int(val_str, 0)
                    self.write_op(0x1, rd=7) # Use R7 as a temporary register
                    self.write_imm(val)
                    self.write_op(0x5, rd=ra, rs=7)
            elif mnemonic in ['JNZ', 'JMP', 'CALL']:
                label = parts[1]
                op = {'JNZ': 0x6, 'JMP': 0x7, 'CALL': 0xD}[mnemonic]
                if label in self.labels:
                    target_ip = self.labels[label]
                    rel = target_ip - (current_ip + 1)
                    self.write_op(op, b=rel & 0xFF)
                else:
                    self.unresolved.append({'ip': current_ip, 'label': label, 'op': op})
                    self.write_op(op, b=0) # Placeholder
            elif mnemonic == 'LOAD':
                rd = int(parts[1][1])
                x = int(parts[2], 0)
                y = int(parts[3], 0)
                self.write_op(0x8, rd=rd)
                self.write_imm(x)
                self.write_imm(y)
            elif mnemonic == 'STORE':
                rs = int(parts[1][1])
                x = int(parts[2], 0)
                y = int(parts[3], 0)
                self.write_op(0x9, rs=rs)
                self.write_imm(x)
                self.write_imm(y)
            elif mnemonic == 'BLIT':
                # BLIT [sx,sy],[dx,dy],w,h
                # Simplified parsing for now
                sx = int(parts[1], 0)
                sy = int(parts[2], 0)
                dx = int(parts[3], 0)
                dy = int(parts[4], 0)
                w = int(parts[5], 0)
                h = int(parts[6], 0)
                self.write_op(0xA)
                self.write_imm(sx)
                self.write_imm(sy)
                self.write_imm(dx)
                self.write_imm(dy)
                self.write_imm(w)
                self.write_imm(h)
            elif mnemonic == 'IN':
                rd = int(parts[1][1])
                port = int(parts[2], 0)
                self.write_op(0xB, rd=rd)
                self.write_imm(port)
            elif mnemonic == 'RET':
                self.write_op(0xE)
            elif mnemonic == 'HALT':
                self.write_op(0xF)

        # Third pass: resolve labels
        for unresolved in self.unresolved:
            label = unresolved['label']
            if label in self.labels:
                target_ip = self.labels[label]
                rel = target_ip - (unresolved['ip'] + 1)

                # Re-write the op with the correct relative address
                op_x = unresolved['ip'] % self.width
                op_y = unresolved['ip'] // self.width + 1

                r, g, _ = self.img.getpixel((op_x, op_y))
                self.img.putpixel((op_x, op_y), (r, g, rel & 0xFF))
            else:
                raise ValueError(f"Undefined label: {label}")

        # Header Row
        self.img.putpixel((0, 0), (80, 80, 80)) # Magic
        self.img.putpixel((1, 0), U24_to_rgb(0)) # C=0
        self.img.putpixel((2, 0), U24_to_rgb(self.code_y)) # K
        self.img.putpixel((3, 0), U24_to_rgb(0)) # EP=0

        self.img.save(out_file)
        print(f"{out_file} created successfully.")

    def get_instruction_size(self, line):
        parts = line.split()
        mnemonic = parts[0].upper()
        if mnemonic in ['MOV', 'OUT', 'IN']:
            return 2
        elif mnemonic == 'CMP':
            if parts[2].startswith('R'):
                return 1
            else:
                return 3 # Emulated CMP with immediate
        elif mnemonic in ['ADD', 'SUB', 'JNZ', 'JMP', 'RET', 'HALT', 'CALL']:
            return 1
        elif mnemonic in ['LOAD', 'STORE']:
            return 3
        elif mnemonic == 'BLIT':
            return 7
        return 0

if __name__ == '__main__':
    import sys
    if len(sys.argv) < 3:
        print("Usage: ppl_asm.py <input.ppla> <output.png>")
        sys.exit(1)

    in_file = sys.argv[1]
    out_file = sys.argv[2]

    asm = Assembler()
    asm.assemble(in_file, out_file)
