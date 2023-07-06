#! /bin/sh

# Exit if any commands fail
set -e

echo "Building programs"

cd programs
cargo build --target x86_64-angeles.json --bin $1
cd ..

cd graphics
cargo build --target x86_64-angeles.json
cd ..

mkdir -p target/programs target/servers
cp target/x86_64-angeles/debug/$1.elf target/programs/current1.elf
cp target/x86_64-angeles/debug/graphics.elf target/servers/graphics.elf

echo "Building kernel"

cd kernel
cargo build --target x86_64-angeles.json 
cd ..

echo "Kernel built successfully"
echo "Creating bootable image"

# Create an empty zeroed out 64MiB image file.
dd if=/dev/zero bs=1M count=0 seek=64 of=target/disk.img
 
# Create a GPT partition table.
parted -s target/disk.img mklabel gpt
 
# Create an ESP partition that spans the whole disk.
parted -s target/disk.img mkpart ESP fat32 2048s 100%
parted -s target/disk.img set 1 esp on

# Build limine-deploy.
make -C limine
 
# Install the Limine BIOS stages onto the image.
./limine/limine-deploy target/disk.img
 
# Mount the loopback device.
# Needs root
USED_LOOPBACK=$(losetup -Pf --show target/disk.img)
 
# Format the ESP partition as FAT32.
# Needs root
mkfs.fat -F 32 ${USED_LOOPBACK}p1
 
# Mount the partition itself.
mkdir -p img_mount
# Needs root
mount ${USED_LOOPBACK}p1 img_mount
 
# Copy the relevant files over.
# Needs root ?
mkdir -p img_mount/EFI/BOOT
# Needs root ??
cp -v target/x86_64-angeles/debug/losangeles.elf limine.cfg limine/limine.sys img_mount/
# Why does this need root ???
cp -v limine/BOOTX64.EFI img_mount/EFI/BOOT/
 
# So it has time to stop being busy
sleep 0.25

# Sync system cache and unmount partition and loopback device.
sync
# Needs root
umount img_mount
# Needs root
losetup -d ${USED_LOOPBACK}

chmod 777 target/disk.img

echo "Build successful"