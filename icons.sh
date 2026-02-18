#!/bin/env sh

mkdir repos
mkdir icons

function shallow_sparse_submodule() {
  # argument : <repository> <clone target> <sparse target> <result copy source under sparse target> <result copy target>
  git clone --depth=1 --sparse --filter=blob:none --no-checkout "$1" "$2"
  git -C "$2" sparse-checkout set "$3"
  git submodule add -f --depth 1 "$1" "$2"
  # git submodule init "$2"
  git submodule update --force --checkout $2
  # git submodule absorbgitdirs "$2"
  mkdir "$5"
  cp "$2/$3/$4" "$5/"
}

#### icon package lucide ####
shallow_sparse_submodule https://github.com/lucide-icons/lucide.git repos/lucide icons '*.svg' icons/lucide

#### icon package lucide-lab ####
shallow_sparse_submodule https://github.com/lucide-icons/lucide-lab.git repos/lucide-lab icons '*.svg' icons/lucide-lab

#### icon package heroicons ####
shallow_sparse_submodule https://github.com/tailwindlabs/heroicons.git repos/heroicons src '24/outline/*.svg' icons/heroicons

#### icon package mdi(material design icons) ####
shallow_sparse_submodule https://github.com/marella/material-design-icons.git repos/mdi svg/outlined '*.svg' icons/mdi

#### icon package material-symbols  ###
shallow_sparse_submodule https://github.com/marella/material-symbols.git repos/material-symbols svg/400/outlined '*.svg' icons/material-symbols

exit
