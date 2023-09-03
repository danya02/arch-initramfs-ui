all: build-rust add-custom-hooks build install

build-rust:
	cd boot-menu && cargo build --release

build:
	mkinitcpio -p ./linux.preset

add-custom-hooks:
	sudo cp -R ./hooks/* /etc/initcpio/

install:
	sudo mkdir -p /boot/efi/EFI/ArchEFI
	sudo cp ./arch-linux.efi /boot/efi/EFI/ArchEFI/arch-linux.efi
