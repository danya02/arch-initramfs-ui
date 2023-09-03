#!/bin/bash
echo "This will create a new UEFI boot menu record pointing at disk /dev/nvme0n1p1 at file "/EFI/ArchEFI/arch-linux.efi". You probably don't want this -- press ^C now to cancel!"
echo "sleeping 30..."
sleep 30

sudo efibootmgr --create --disk /dev/nvme0n1p1 --loader "\EFI\ArchEFI\arch-linux.efi" --label "Arch Linux Boot Menu"
