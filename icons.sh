#!/bin/env sh

mkdir repos
mkdir icons
cd repos

mkdir ../icons/lucide
mkdir lucide
cd lucide
git fetch
git pull
cp ./icons/*.svg ../../icons/lucide/
cd ..

mkdir ../icons/lucide-lab
mkdir lucide-lab
cd lucide-lab
git fetch
git pull
cp ./icons/*.svg ../../icons/lucide-lab
cd ..

mkdir ../icons/heroicons
mkdir heroicons
cd heroicons
git init
git remote add origin https://github.com/tailwindlabs/heroicons
git sparse-checkout set src
git pull origin master --depth 1 
cp ./src/24/outline/*.svg ../../icons/heroicons/
# for LINE in $(find ./heroicons/src/24/outline/*.svg); do
#   # tp=$(basename $LINE)
#   name=$(basename $LINE)
#   # cp "$LINE/$FILE" "../icons/heroicons/$name$tp.svg"
#   cp "$LINE" "../icons/heroicons/$name"
# done


mkdir ../icons/mdi
mkdir mdi
cd mdi
git init
git remote add origin https://github.com/marella/material-design-icons.git
git sparse-checkout set svg
git pull origin main --depth 1

cp ./svg/outlined/* ../../icons/mdi/
cd ..
# for LINE in $(find ./mdi/svg/* -type d); do
#   tp=$(basename $LINE)
#   if [ "$tp" = "filled" ]; then
#     tp=""
#   else
#     tp="-$tp"
#   fi
#   for FILE in $(ls -1 $LINE); do
#     name=$(basename $FILE .svg)
#     # name=${name//_/-}
#     cp "$LINE/$FILE" "../icons/mdi/$name$tp.svg"
#   done
# done

# git remote add origin https://github.com/google/material-design-icons.git
# git sparse-checkout set src
# git pull origin master --depth 1
# cd ..

# shopt -s globstar
# mkdir ../icons/material-design-icons
# for LINE in $(ls -1 material-design-icons/src/*/*/*/24px.svg); do
#   tp=$(basename $(dirname $LINE))
#   tp=${tp:13}
#   if [ -n "$tp" ]; then tp="-$tp"; fi
#   name=$(basename $(dirname $(dirname $LINE)))
#   name=${name//_/-}
#   cp $LINE "../icons/material-design-icons/$name$tp.svg"
# done


mkdir ../icons/material-symbols
mkdir material-symbols
git init
git remote add origin https://github.com/marella/material-design-icons.git
git sparse-checkout set scripts
git pull origin main --depth 1
npm install @material-design-icons/scripts@latest --save-dev
mkdir svg
npx @material-design-icons/scripts download svg --symbols

# for LINE in $(find ./material-symbols/svg/* -type d); do
#   tp=$(basename $LINE)
#   if [ "$tp" = "outlined" ]; then
#     tp=""
#   else
#     tp="-$tp"
#   fi
#   for FILE in $(ls -1 $LINE); do
#     name=$(basename $FILE .svg)
#     # name=${name//_/-}
#     cp "$LINE/$FILE" "../icons/material-symbols/$name$tp.svg"
#   done
# done
cp ./svg/outlined/*.svg ../../icons/material-symbols/

# cd ..
# zstd --train ./icons/*/*.svg -o ./icons/icons.zst.dict --maxdict=2K
# openssl dgst -sha256 -binary ./icons/icons.zst.dict > ./icons/icons.zst.dict.sha256
# # DICTHASH=$(openssl dgst -sha256 -binary ./icons/icons.zst.dict)
# # for LINE in $(ls -1 ./icons/*/*.svg); do
# #   echo -en '\x5e\x2a\x4d\x18\x20\x00\x00\x00' > "$LINE.dcz" && echo $DICTHASH >> "$LINE.dcz" && zstd -D ./icons/icons.zst.dict -f -c -19 $LINE >> "$LINE.dcz"
# # done
# # cd ..
# zstd -D ./icons/icons.zst.dict -f ./icons/*/*.svg -19

# cp ./icons/icons.zst.dict ../assets/icons/
# cp ./icons/icons.zst.dict.sha256 ../assets/icons/
# for ICON in $(find ./icons/* -type d); do
#   ICON=$(basename $ICON)
#   mkdir -p ../assets/icons/$ICON
#   mv ./icons/$ICON/*.zst ../assets/icons/$ICON
# done

# cd icons
# zstd -D ./icons.zst.dict icons.tar
# tar cf ./icons.tar.zst $(find ./* -type d | cut -c 3-) --zstd
# cp ./icons.tar.zst ../../assets/icons/

# tar cf ./icons.tar $(find ./* -type d | cut -c 3-)
# cp ./icons.tar.zst ../../assets/icons/

# zstd -D ./icons.zst.dict -19 -f ./*/*.svg 
# tar cf ./icons.zst.tar */*.svg.zst
# rm */*.svg.zst

# cd ..
