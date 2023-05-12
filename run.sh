#! /bin/sh

AUDIO=alsa

# added `-serial stdio` for debugging
qemu-system-x86_64 \
-serial stdio \
-m 512m \
-cpu qemu64 \
-hda disk.img \
-audio "${AUDIO},model=ac97" \
-vga cirrus # Cirrus Logic GD5446 Video card