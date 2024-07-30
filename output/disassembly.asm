bits 16
mov bp, word 256
mov dx, word 0
mov cx, word 0
mov word [bp +0], cx
mov word [bp +2], dx
mov byte [bp +3], byte 255
add bp, byte 4
add cx, byte 1
cmp cx, byte 64
jne byte 235
add dx, byte 1
cmp dx, byte 64
jne byte 224
