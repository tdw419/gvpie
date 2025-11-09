#!/usr/bin/env python3
import sys, math
from PIL import Image
import json

U24 = lambda r,g,b: (r<<16)|(g<<8)|b

class VM:
    def __init__(self, img: Image.Image):
        self.img = img.convert('RGB')
        self.w, self.h = self.img.size
        self.R = [0]*8
        self.ZF = 0
        self.HF = 0
        self.IP = 0  # code-cell index, linearized over code band
        self.stack = []
        self.header = self._decode_header()
        self.code_base = (0, self.header['C']+1)  # (x0, y0)
        self.trace = []

    def _px(self, x, y):
        r,g,b = self.img.getpixel((x,y))
        return r,g,b

    def _put(self, x, y, v):
        r = (v>>16)&0xFF; g=(v>>8)&0xFF; b=v&0xFF
        self.img.putpixel((x,y),(r,g,b))

    def _decode_header(self):
        r0 = U24(*self._px(0,0))
        C   = U24(*self._px(1,0))
        K   = U24(*self._px(2,0))
        EP  = U24(*self._px(3,0))
        if r0 != U24(80,80,80):
            raise ValueError('Bad magic')
        return {'C':C, 'K':K, 'EP':EP}

    def _code_xy(self, idx):
        # Linear index → (x,y) within the code band
        row_len = self.w
        y = idx//row_len
        x = idx%row_len
        return x, self.code_base[1] + y

    def _cell(self, idx):
        x,y = self._code_xy(idx)
        return self._px(x,y)

    def step(self):
        r,g,b = self._cell(self.IP)
        op = r & 0xF
        rd = (g>>4)&0x7; rs = g&0x7
        ip0 = self.IP
        self.IP += 1

        def next_u24():
            r,g,b = self._cell(self.IP)
            self.IP += 1
            return U24(r,g,b)

        def next_i8():
            # rel8 packed in B of the *current* opcode cell for compactness
            return (b-256) if b>127 else b

        if op==0x0: pass
        elif op==0x1: self.R[rd]=next_u24()
        elif op==0x2: self.R[rd]=self.R[rs]
        elif op==0x3:
            self.R[rd]=(self.R[rd]+self.R[rs])&0xFFFFFF; self.ZF=(self.R[rd]==0)
        elif op==0x4:
            self.R[rd]=(self.R[rd]-self.R[rs])&0xFFFFFF; self.ZF=(self.R[rd]==0)
        elif op==0x5:
            self.ZF = int(self.R[rd]==self.R[rs])
        elif op==0x6:
            rel = next_i8();
            if self.ZF==0: self.IP = (self.IP+rel)
        elif op==0x7:
            rel = next_i8(); self.IP = (self.IP+rel)
        elif op==0x8:
            x = next_u24()%self.w; y = next_u24()%self.h
            self.R[rd] = U24(*self._px(x,y))
        elif op==0x9:
            x = next_u24()%self.w; y = next_u24()%self.h
            self._put(x,y,self.R[rs])
        elif op==0xA:
            sx=next_u24()%self.w; sy=next_u24()%self.h; dx=next_u24()%self.w; dy=next_u24()%self.h
            w=next_u24()%self.w; h=next_u24()%self.h
            for yy in range(h):
                for xx in range(w):
                    self._put((dx+xx)%self.w,(dy+yy)%self.h, U24(*self._px((sx+xx)%self.w,(sy+yy)%self.h)))
        elif op==0xB:
            port = next_u24()&0xFF
            self.R[rd] = self._io_read(port)
        elif op==0xC:
            port = next_u24()&0xFF
            self._io_write(port, self.R[rs])
        elif op==0xD:
            rel = next_i8(); self.stack.append(self.IP); self.IP = (self.IP+rel)
        elif op==0xE:
            self.IP = self.stack.pop() if self.stack else self.IP
        elif op==0xF:
            self.HF=1
        else:
            raise RuntimeError('bad opcode')

        self.trace.append((ip0,op,rd,rs,self.R[:],self.ZF))

    def run(self, max_steps=100000):
        self.IP = self.header['EP']
        steps=0
        while not self.HF and steps<max_steps:
            self.step(); steps+=1
        return steps

    def _io_read(self, port):
        if port==0x20: # health probe demo
            # Return 0x1 if image has any nonzero pixel in Data band
            y0 = 1+self.header['C']+self.header['K']
            for y in range(y0,self.h):
                for x in range(self.w):
                    if any(self._px(x,y)): return 1
            return 0
        elif port==0x01: # monotonic time stub
            return 12345
        elif port==0x02: # random stub
            return 0xABCDE & 0xFFFFFF
        elif port==0x11: # getchar none
            return 0xFFFFFF
        return 0

    def _io_write(self, port, val):
        if port==0x10:
            ch = val & 0xFF
            sys.stdout.write(chr(ch)); sys.stdout.flush()

if __name__=='__main__':
    if len(sys.argv)<3:
        print('usage: ppl_run.py in.png out.png'); sys.exit(1)
    img = Image.open(sys.argv[1])
    vm = VM(img)
    vm.run()
    vm.img.save(sys.argv[2])

    with open('trace.json', 'w') as f:
        json.dump(vm.trace, f, indent=2)
