ldxdw r5, [r1+24]
ldxdw r4, [r1+16]
mov64 r1, r10
add64 r1, -16
mov64 r2, -1
mov64 r3, -1
call -619746029
ldxdw r1, [r10-8]
ldxdw r2, [r10-16]
or64 r2, r1
lddw r0, 103079215104
jeq r2, r0, +1
mov64 r0, 0
exit
