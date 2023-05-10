#! /bin/sh

if [ ! -f "target/x86_64-angeles/debug/losangeles.elf" ]; then
    echo "Building kernel"

    cd kernel
    cargo build --target x86_64-angeles.json
    cd ..

    echo "Kernel built successfully"
fi

echo "Creating bootable image"

# Create an empty zeroed out 64MiB image file.
dd if=/dev/zero bs=1M count=0 seek=64 of=disk.img
 
# Create a GPT partition table.
parted -s disk.img mklabel gpt
 
# Create an ESP partition that spans the whole disk.
parted -s disk.img mkpart ESP fat32 2048s 100%
parted -s disk.img set 1 esp on

# Build limine-deploy.
make -C limine
 
# Install the Limine BIOS stages onto the image.
./limine/limine-deploy disk.img
 
# Mount the loopback device.
USED_LOOPBACK=$(losetup -Pf --show disk.img)
 
# Format the ESP partition as FAT32.
mkfs.fat -F 32 ${USED_LOOPBACK}p1 || { echo "Permission denied"; exit 1; }
 
# Mount the partition itself.
mkdir -p img_mount
mount ${USED_LOOPBACK}p1 img_mount
 
# Copy the relevant files over.
mkdir -p img_mount/EFI/BOOT
cp -v target/x86_64-angeles/debug/losangeles.elf limine.cfg limine/limine.sys img_mount/
cp -v limine/BOOTX64.EFI img_mount/EFI/BOOT/
 
# So it has time to stop being busy
sleep 0.25

# Sync system cache and unmount partition and loopback device.
sync
umount img_mount
losetup -d ${USED_LOOPBACK}

echo "Build successful"