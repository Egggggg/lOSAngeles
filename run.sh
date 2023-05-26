#! /bin/sh

AUDIO=alsa

# added `-serial stdio` for debugging
qemu-system-x86_64 \
-S -s \
-serial stdio \
-m 512m \
-cpu qemu64 \
-hda target/disk.img \
-vga cirrus # Cirrus Logic GD5446 Video card
# -audio "${AUDIO},model=ac97" \