#!/usr/bin/env bash

image=$1

# gblur
a=20
b=2

# boxblur
c=1
d=1

# alpha
e=1

ffmpeg -y -i $image -vf "gblur=sigma=$a:steps=$b,boxblur=$c:$d,colorchannelmixer=aa=$e" background.png
