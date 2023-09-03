all: add-custom-hooks build install

build:
	mkinitcpio -p ./linux.preset

add-custom-hooks:
	sudo cp -R ./hooks/* /etc/initcpio/

install:
	sudo mkdir -p /boot/efi/EFI/ArchEFI
	sudo cp ./arch-linux.efi /boot/efi/EFI/ArchEFI/arch-linux.efi
