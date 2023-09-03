#!/bin/bash

# https://aur.archlinux.org/packages/fbterm#comment-926676

rm -rf fbterm/
git clone https://aur.archlinux.org/fbterm.git
cd fbterm
makepkg -o
cd src
cd fbterm-1.7
mv Makefile Makefile.old
sed "s/SUBDIRS = src im terminfo doc/SUBDIRS = src im doc/" ./Makefile.old > Makefile

cd ..
cd ..

makepkg -e
makepkg -i
cd ..

